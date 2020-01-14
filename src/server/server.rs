use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs, thread};

use async_std::task::block_on;
use crossbeam_channel::select;
use futures::Future;
use grpcio::{ChannelBuilder, EnvBuilder, Environment, RpcContext, ServerBuilder, UnarySink};
use log::*;
use protobuf::Message;
use raft::eraftpb::{ConfChange, ConfChangeType, Entry, EntryType, Message as RaftMessage};
use stringreader::StringReader;
use tantivy::collector::{Count, FacetCollector, MultiCollector, TopDocs};
use tantivy::merge_policy::LogMergePolicy;
use tantivy::query::{QueryParser, TermQuery};
use tantivy::schema::{Field, FieldType, IndexRecordOption, Schema};
use tantivy::{Document, Index, IndexWriter, Term};

use crate::client::client::{create_client, Clerk};
use crate::proto::indexpb_grpc::{self, Index as IndexService, IndexClient};
use crate::proto::indexrpcpb::{
    ApplyReq, BulkDeleteReq, BulkDeleteResp, BulkPutReq, BulkPutResp, CommitReq, CommitResp,
    ConfChangeReq, DeleteReq, DeleteResp, GetReq, GetResp, JoinReq, LeaveReq, MergeReq, MergeResp,
    MetricsReq, MetricsResp, PeersReq, PeersResp, ProbeReq, ProbeResp, PutReq, PutResp, RaftDone,
    ReqType, RespErr, RollbackReq, RollbackResp, SchemaReq, SchemaResp, SearchReq, SearchResp,
};
use crate::server::metrics::Metrics;
use crate::server::peer::PeerMessage;
use crate::server::{peer, util};
use crate::tokenizer::tokenizer_initializer::TokenizerInitializer;
use crate::util::search_result::{ScoredNamedFieldDocument, SearchResult};
use crate::util::signal::sigterm_channel;

struct NotifyArgs(u64, String, RespErr);

#[derive(Clone)]
pub struct IndexServer {
    id: u64,
    peers: Arc<Mutex<HashMap<u64, IndexClient>>>,
    peers_addr: Arc<Mutex<HashMap<u64, String>>>,
    rf_message_ch: SyncSender<PeerMessage>,
    notify_ch_map: Arc<Mutex<HashMap<u64, SyncSender<NotifyArgs>>>>,
    index: Arc<Index>,
    index_writer: Arc<Mutex<IndexWriter>>,
    metrics: Arc<Mutex<Metrics>>,
}

impl IndexServer {
    pub fn start_server(
        id: u64,
        host: &str,
        port: u16,
        peers_addr: HashMap<u64, String>,
        data_directory: &str,
        schema_file: &str,
        tokenizer_file: &str,
        indexer_threads: usize,
        indexer_memory_size: usize,
    ) {
        let mut peers = HashMap::new();
        peers.insert(id, create_client(&format!("{}:{}", host, port)));
        for (peer_id, peer_addr) in peers_addr.iter() {
            peers.insert(*peer_id, create_client(peer_addr));
        }

        let raft_path = Path::new(data_directory).join(Path::new("raft"));
        fs::create_dir_all(&raft_path).unwrap_or_default();

        let schema_content = fs::read_to_string(schema_file).unwrap();
        let schema: Schema =
            serde_json::from_str(&schema_content).expect("error while reading json");

        let index_path = Path::new(data_directory).join(Path::new("index"));
        let index = if index_path.exists() {
            Index::open_in_dir(index_path.to_str().unwrap()).unwrap()
        } else {
            fs::create_dir_all(&index_path).unwrap_or_default();
            Index::create_in_dir(index_path.to_str().unwrap(), schema).unwrap()
        };

        if tokenizer_file != "" {
            debug!("{}", tokenizer_file);
            let tokenizer_content = fs::read_to_string(tokenizer_file).unwrap();
            let mut tokenizer_initializer = TokenizerInitializer::new();
            tokenizer_initializer.init(index.tokenizers(), tokenizer_content.as_str());
            // TokenizerInitializer::init(index.tokenizers(), tokenizer_content.as_str())
        }

        let index_writer = if indexer_threads > 0 {
            index
                .writer_with_num_threads(indexer_threads, indexer_memory_size)
                .unwrap()
        } else {
            index.writer(indexer_memory_size).unwrap()
        };
        index_writer.set_merge_policy(Box::new(LogMergePolicy::default()));
        // index_writer.set_merge_policy(Box::new(NoMergePolicy));

        let (rf_sender, rf_receiver) = mpsc::sync_channel(100);
        let (rpc_sender, rpc_receiver) = mpsc::sync_channel(100);
        let (apply_sender, apply_receiver) = mpsc::sync_channel(100);

        let peers_id = peers.keys().map(|id| *id).collect();

        let mut index_server = IndexServer {
            id,
            peers: Arc::new(Mutex::new(peers)),
            peers_addr: Arc::new(Mutex::new(peers_addr)),
            rf_message_ch: rf_sender,
            notify_ch_map: Arc::new(Mutex::new(HashMap::new())),
            index: Arc::new(index),
            index_writer: Arc::new(Mutex::new(index_writer)),
            metrics: Arc::new(Mutex::new(Metrics::new(id))),
        };

        index_server.async_rpc_sender(rpc_receiver);
        index_server.async_applier(apply_receiver);

        let env = Arc::new(Environment::new(10));
        let service = indexpb_grpc::create_index(index_server.clone());
        let mut server = ServerBuilder::new(env)
            .register_service(service)
            .bind(host, port)
            .build()
            .unwrap_or_else(|e| {
                panic!("build server error: {}", e);
            });

        server.start();
        for &(ref host, port) in server.bind_addrs() {
            info!("listening on {}:{}", host, port);
        }

        let peer = peer::Peer::new(id, apply_sender, peers_id);
        peer::Peer::activate(peer, rpc_sender, rf_receiver);

        let mut servers: Vec<IndexClient> = Vec::new();
        for (_, value) in index_server.peers.clone().lock().unwrap().iter() {
            servers.push(value.clone());
        }

        let client_id = rand::random();
        let mut client = Clerk::new(&servers, client_id);
        client.join_with_retry(id, host, port, 10, Duration::from_secs(3));

        // Wait for signals for termination (SIGINT, SIGTERM).
        let sigterm_receiver = sigterm_channel().unwrap();
        loop {
            select! {
                recv(sigterm_receiver) -> _ => {
                    info!("stopping on {}:{}", host, port);
                    let _ = server.shutdown().wait();
                    break;
                }
            }
        }
    }

    fn async_rpc_sender(&mut self, receiver: Receiver<RaftMessage>) {
        let l = self.peers.clone();
        thread::spawn(move || loop {
            match receiver.recv() {
                Ok(m) => {
                    let peers = l.lock().unwrap();
                    let op = peers.get(&m.to);
                    if let Some(c) = op {
                        let client = c.clone();
                        thread::spawn(move || {
                            client.raft(&m).unwrap_or_else(|e| {
                                error!("send raft msg to {} failed: {:?}", m.to, e);
                                RaftDone::new()
                            });
                        });
                    }
                }
                Err(_) => (),
            }
        });
    }

    fn apply(&mut self, req: &ApplyReq) -> (RespErr, String) {
        let (sh, rh) = mpsc::sync_channel(0);
        {
            let mut map = self.notify_ch_map.lock().unwrap();
            map.insert(req.get_client_id(), sh);
        }
        self.rf_message_ch
            .send(PeerMessage::Propose(req.write_to_bytes().unwrap_or_else(
                |e| {
                    panic!("request write to bytes error: {:?}", e);
                },
            )))
            .unwrap_or_else(|e| {
                error!("send propose to raft error: {:?}", e);
            });
        // TODO: consider appropriate timeout value
        return match rh.recv_timeout(Duration::from_millis(60000)) {
            Ok(args) => (args.2, args.1),
            Err(e) => {
                {
                    let mut map = self.notify_ch_map.lock().unwrap();
                    map.remove(&req.get_client_id());
                }

                let mut ret = HashMap::new();
                ret.insert("error", format!("{:?}", e));
                match e {
                    mpsc::RecvTimeoutError::Timeout => {
                        (RespErr::ErrTimeout, serde_json::to_string(&ret).unwrap())
                    }
                    mpsc::RecvTimeoutError::Disconnected => (
                        RespErr::ErrDisconnected,
                        serde_json::to_string(&ret).unwrap(),
                    ),
                }
            }
        };
    }

    // TODO: check duplicate request.
    fn async_applier(&mut self, apply_receiver: Receiver<Entry>) {
        let notify_ch_map = self.notify_ch_map.clone();
        let peers = self.peers.clone();
        let peers_addr = self.peers_addr.clone();
        let index = self.index.clone();
        let index_writer = self.index_writer.clone();
        let metrics = self.metrics.clone();

        thread::spawn(move || loop {
            match apply_receiver.recv() {
                Ok(e) => match e.get_entry_type() {
                    EntryType::EntryNormal => {
                        let result: NotifyArgs;
                        let req: ApplyReq = util::parse_data(e.get_data());
                        let client_id = req.get_client_id();
                        if e.data.len() > 0 {
                            result = Self::apply_entry(
                                e.term,
                                &req,
                                peers.clone(),
                                peers_addr.clone(),
                                index.clone(),
                                index_writer.clone(),
                                metrics.clone(),
                            );
                            debug!("{:?}: {:?}", result.2, req);
                        } else {
                            result = NotifyArgs(0, String::from(""), RespErr::ErrWrongLeader);
                            debug!("{:?}", req);
                        }
                        let mut map = notify_ch_map.lock().unwrap();
                        if let Some(s) = map.get(&client_id) {
                            s.send(result).unwrap_or_else(|e| {
                                error!("notify apply result error: {:?}", e);
                            });
                        }
                        map.remove(&client_id);
                    }
                    EntryType::EntryConfChange => {
                        let result = NotifyArgs(0, String::from(""), RespErr::OK);
                        let cc: ConfChange = util::parse_data(e.get_data());
                        let mut map = notify_ch_map.lock().unwrap();
                        if let Some(s) = map.get(&cc.get_node_id()) {
                            s.send(result).unwrap_or_else(|e| {
                                error!("notify apply result error: {:?}", e);
                            });
                        }
                        map.remove(&cc.get_node_id());
                    }
                },
                Err(_) => (),
            }
        });
    }

    fn apply_entry(
        term: u64,
        req: &ApplyReq,
        peers: Arc<Mutex<HashMap<u64, IndexClient>>>,
        peers_addr: Arc<Mutex<HashMap<u64, String>>>,
        index: Arc<Index>,
        index_writer: Arc<Mutex<IndexWriter>>,
        metrics: Arc<Mutex<Metrics>>,
    ) -> NotifyArgs {
        match req.req_type {
            ReqType::Join => {
                metrics.lock().unwrap().inc_request_count("join");

                let mut prs = peers.lock().unwrap();
                let env = Arc::new(EnvBuilder::new().build());
                let ch = ChannelBuilder::new(env).connect(&req.get_join_req().peer_addr);
                prs.insert(req.get_join_req().peer_id, IndexClient::new(ch));

                let mut prs_addr = peers_addr.lock().unwrap();
                prs_addr.insert(
                    req.get_join_req().peer_id,
                    req.get_join_req().peer_addr.clone(),
                );

                NotifyArgs(term, String::from(""), RespErr::OK)
            }
            ReqType::Leave => {
                metrics.lock().unwrap().inc_request_count("leave");

                let mut prs = peers.lock().unwrap();
                prs.remove(&req.get_leave_req().peer_id);

                let mut prs_addr = peers_addr.lock().unwrap();
                prs_addr.remove(&req.get_leave_req().peer_id);

                NotifyArgs(term, String::from(""), RespErr::OK)
            }
            ReqType::Put => {
                metrics.lock().unwrap().inc_request_count("put");

                let doc_id_field = index.schema().get_field("_id").unwrap();

                let doc = index
                    .schema()
                    .parse_document(req.get_put_req().get_doc())
                    .unwrap();
                let doc_id = doc.get_first(doc_id_field).unwrap();

                // delete doc
                index_writer
                    .lock()
                    .unwrap()
                    .delete_term(Term::from_field_text(doc_id_field, doc_id.text().unwrap()));

                // add doc
                let opstamp = index_writer.lock().unwrap().add_document(doc);

                let mut ret = HashMap::new();
                ret.insert("opstamp", opstamp);

                NotifyArgs(term, serde_json::to_string(&ret).unwrap(), RespErr::OK)
            }
            ReqType::Delete => {
                metrics.lock().unwrap().inc_request_count("delete");

                let opstamp = index_writer
                    .lock()
                    .unwrap()
                    .delete_term(Term::from_field_text(
                        index.schema().get_field("_id").unwrap(),
                        req.get_delete_req().get_doc_id(),
                    ));

                let mut ret = HashMap::new();
                ret.insert("opstamp", opstamp);

                NotifyArgs(term, serde_json::to_string(&ret).unwrap(), RespErr::OK)
            }
            ReqType::BulkPut => {
                metrics.lock().unwrap().inc_request_count("bulk_put");

                let doc_id_field = index.schema().get_field("_id").unwrap();
                let mut opstamp = 0;

                let mut reader =
                    BufReader::new(StringReader::new(req.get_bulk_put_req().get_docs()));
                let mut line = String::new();
                while reader.read_line(&mut line).unwrap() > 0 {
                    let doc = index.schema().parse_document(&line).unwrap();
                    let doc_id = doc.get_first(doc_id_field).unwrap();

                    // delete doc
                    index_writer
                        .lock()
                        .unwrap()
                        .delete_term(Term::from_field_text(doc_id_field, doc_id.text().unwrap()));

                    // add doc
                    opstamp = index_writer.lock().unwrap().add_document(doc);

                    line.clear();
                }

                let mut ret = HashMap::new();
                ret.insert("opstamp", opstamp);

                NotifyArgs(term, serde_json::to_string(&ret).unwrap(), RespErr::OK)
            }
            ReqType::BulkDelete => {
                metrics.lock().unwrap().inc_request_count("bulk_delete");

                let doc_id_field = index.schema().get_field("_id").unwrap();
                let mut opstamp = 0;

                let mut reader =
                    BufReader::new(StringReader::new(req.get_bulk_delete_req().get_docs()));
                let mut line = String::new();
                while reader.read_line(&mut line).unwrap() > 0 {
                    let doc = index.schema().parse_document(&line).unwrap();
                    let doc_id = doc.get_first(doc_id_field).unwrap();

                    opstamp = index_writer
                        .lock()
                        .unwrap()
                        .delete_term(Term::from_field_text(doc_id_field, doc_id.text().unwrap()));

                    line.clear();
                }

                let mut ret = HashMap::new();
                ret.insert("opstamp", opstamp);

                NotifyArgs(term, serde_json::to_string(&ret).unwrap(), RespErr::OK)
            }
            ReqType::Commit => {
                metrics.lock().unwrap().inc_request_count("commit");

                match index_writer.lock().unwrap().commit() {
                    Ok(opstamp) => {
                        info!("commit succeeded");

                        let mut ret = HashMap::new();
                        ret.insert("opstamp", opstamp);

                        NotifyArgs(term, serde_json::to_string(&ret).unwrap(), RespErr::OK)
                    }
                    Err(e) => {
                        error!("commit failed: {:?}", e);

                        // TODO: rollback index

                        let mut ret = HashMap::new();
                        ret.insert("error", format!("{:?}", e));

                        NotifyArgs(
                            term,
                            serde_json::to_string(&ret).unwrap(),
                            RespErr::ErrCommitFailed,
                        )
                    }
                }
            }
            ReqType::Rollback => {
                metrics.lock().unwrap().inc_request_count("rollback");

                match index_writer.lock().unwrap().rollback() {
                    Ok(opstamp) => {
                        info!("rollback succeed");

                        let mut ret = HashMap::new();
                        ret.insert("opstamp", opstamp);

                        NotifyArgs(term, serde_json::to_string(&ret).unwrap(), RespErr::OK)
                    }
                    Err(e) => {
                        error!("rollback failed: {:?}", e);

                        let mut ret = HashMap::new();
                        ret.insert("error", format!("{:?}", e));

                        NotifyArgs(
                            term,
                            serde_json::to_string(&ret).unwrap(),
                            RespErr::ErrRollbackFailed,
                        )
                    }
                }
            }
            ReqType::Merge => {
                metrics.lock().unwrap().inc_request_count("merge");

                let segments = index.searchable_segment_ids().unwrap();

                // check segments length
                if segments.len() <= 0 {
                    // do not merge segments
                    let mut ret = HashMap::new();
                    ret.insert("segments", segments);

                    return NotifyArgs(term, serde_json::to_string(&ret).unwrap(), RespErr::OK);
                }

                let merge_future = index_writer.lock().unwrap().merge(&segments);
                match block_on(merge_future) {
                    Ok(segment_meta) => {
                        info!("merge succeed: {:?}", segment_meta);

                        let mut ret = HashMap::new();
                        ret.insert("segment_meta", segment_meta);
                        NotifyArgs(term, serde_json::to_string(&ret).unwrap(), RespErr::OK)
                    }
                    Err(e) => {
                        error!("merge failed: {:?}", e);

                        let mut ret = HashMap::new();
                        ret.insert("error", format!("{:?}", e));

                        NotifyArgs(
                            term,
                            serde_json::to_string(&ret).unwrap(),
                            RespErr::ErrMergeFailed,
                        )
                    }
                }
            }
        }
    }
}

impl IndexService for IndexServer {
    fn raft(&mut self, ctx: RpcContext, req: RaftMessage, sink: UnarySink<RaftDone>) {
        self.metrics.lock().unwrap().inc_request_count("raft");

        self.rf_message_ch
            .send(PeerMessage::Message(req.clone()))
            .unwrap_or_else(|e| {
                error!("send message to raft error: {:?}", e);
            });
        let resp = RaftDone::new();

        ctx.spawn(
            sink.success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", req, e)),
        )
    }

    fn raft_conf_change(&mut self, ctx: RpcContext, req: ConfChangeReq, sink: UnarySink<RaftDone>) {
        debug!("request: {:?}", req);

        self.metrics
            .lock()
            .unwrap()
            .inc_request_count("raft_conf_change");

        let cc = req.cc.clone().unwrap();
        let mut resp = RaftDone::new();
        let mut apply_req = ApplyReq::new();

        match cc.change_type {
            ConfChangeType::AddNode | ConfChangeType::AddLearnerNode => {
                apply_req.set_req_type(ReqType::Join);
                let mut join_req = JoinReq::new();
                join_req.set_client_id(cc.get_node_id());
                join_req.set_peer_id(cc.get_node_id());
                join_req.set_peer_addr(format!("{}:{}", req.ip, req.port));
                apply_req.set_join_req(join_req);
            }
            ConfChangeType::RemoveNode => {
                apply_req.set_req_type(ReqType::Leave);
                let mut leave_req = LeaveReq::new();
                leave_req.set_client_id(cc.get_node_id());
                leave_req.set_peer_id(cc.get_node_id());
                leave_req.set_peer_addr(format!("{}:{}", req.ip, req.port));
                apply_req.set_leave_req(leave_req);
            }
        }
        let (err, _) = self.apply(&apply_req);
        match err {
            RespErr::OK => {
                let (sh, rh) = mpsc::sync_channel(0);
                {
                    let mut map = self.notify_ch_map.lock().unwrap();
                    map.insert(cc.get_node_id(), sh);
                }
                self.rf_message_ch
                    .send(PeerMessage::ConfChange(cc.clone()))
                    .unwrap();
                match rh.recv_timeout(Duration::from_millis(1000)) {
                    Ok(_) => resp.set_err(RespErr::OK),
                    Err(_) => resp.set_err(RespErr::ErrWrongLeader),
                }
            }
            _ => resp.set_err(RespErr::ErrWrongLeader),
        }

        debug!("response: {:?}", resp);

        ctx.spawn(
            sink.success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", req, e)),
        )
    }

    fn probe(&mut self, ctx: RpcContext, req: ProbeReq, sink: UnarySink<ProbeResp>) {
        debug!("request: {:?}", req);

        self.metrics.lock().unwrap().inc_request_count("probe");

        let mut ret = HashMap::new();
        ret.insert("health", "OK");

        let mut resp = ProbeResp::new();
        resp.set_err(RespErr::OK);
        resp.set_value(serde_json::to_string(&ret).unwrap());

        debug!("response: {:?}", resp);

        ctx.spawn(
            sink.success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", req, e)),
        )
    }

    fn peers(&mut self, ctx: RpcContext, req: PeersReq, sink: UnarySink<PeersResp>) {
        debug!("request: {:?}", req);

        self.metrics.lock().unwrap().inc_request_count("peers");

        let mut resp = PeersResp::new();
        resp.set_err(RespErr::OK);
        resp.set_value(serde_json::to_string(&self.peers_addr.lock().unwrap().clone()).unwrap());

        debug!("response: {:?}", resp);

        ctx.spawn(
            sink.success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", req, e)),
        )
    }

    fn metrics(&mut self, ctx: RpcContext, req: MetricsReq, sink: UnarySink<MetricsResp>) {
        debug!("request: {:?}", req);

        self.metrics.lock().unwrap().inc_request_count("metrics");

        let mut resp = MetricsResp::new();
        resp.set_err(RespErr::OK);
        resp.set_value(self.metrics.lock().unwrap().get_metrics());

        debug!("response: {:?}", resp);

        ctx.spawn(
            sink.success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", req, e)),
        )
    }

    fn get(&mut self, ctx: RpcContext, req: GetReq, sink: UnarySink<GetResp>) {
        debug!("request: {:?}", req);

        self.metrics.lock().unwrap().inc_request_count("get");

        let t = Term::from_field_text(
            self.index.schema().get_field("_id").unwrap(),
            req.get_doc_id(),
        );
        let tq = TermQuery::new(t, IndexRecordOption::Basic);
        let searcher = self.index.reader().unwrap().searcher();
        let top_docs = searcher.search(&tq, &TopDocs::with_limit(10)).unwrap();
        let mut doc = Document::default();
        for (_score, doc_address) in top_docs {
            doc = searcher.doc(doc_address).unwrap();
        }
        let named_doc = self.index.schema().to_named_doc(&doc);

        let mut resp = GetResp::new();
        resp.set_err(RespErr::OK);
        resp.set_value(serde_json::to_string(&named_doc).unwrap());

        debug!("response: {:?}", resp);

        ctx.spawn(
            sink.success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", req, e)),
        )
    }

    fn put(&mut self, ctx: RpcContext, req: PutReq, sink: UnarySink<PutResp>) {
        debug!("request: {:?}", req);

        let mut apply_req = ApplyReq::new();
        apply_req.set_client_id(req.get_client_id());
        apply_req.set_req_type(ReqType::Put);
        apply_req.set_put_req(req.to_owned());

        let (err, ret) = Self::apply(self, &apply_req);
        let mut resp = PutResp::new();
        resp.set_err(err);
        resp.set_value(ret);

        debug!("response: {:?}", resp);

        ctx.spawn(
            sink.success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", apply_req, e)),
        )
    }

    fn delete(&mut self, ctx: RpcContext, req: DeleteReq, sink: UnarySink<DeleteResp>) {
        debug!("request: {:?}", req);

        let mut apply_req = ApplyReq::new();
        apply_req.set_client_id(req.get_client_id());
        apply_req.set_req_type(ReqType::Delete);
        apply_req.set_delete_req(req.to_owned());

        let (err, ret) = Self::apply(self, &apply_req);
        let mut resp = DeleteResp::new();
        resp.set_err(err);
        resp.set_value(ret);

        debug!("response: {:?}", resp);

        ctx.spawn(
            sink.success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", apply_req, e)),
        )
    }

    fn bulk_put(&mut self, ctx: RpcContext, req: BulkPutReq, sink: UnarySink<BulkPutResp>) {
        debug!("request: {:?}", req);

        let mut apply_req = ApplyReq::new();
        apply_req.set_client_id(req.get_client_id());
        apply_req.set_req_type(ReqType::BulkPut);
        apply_req.set_bulk_put_req(req.to_owned());

        let (err, ret) = Self::apply(self, &apply_req);
        let mut resp = BulkPutResp::new();
        resp.set_err(err);
        resp.set_value(ret);

        debug!("response: {:?}", resp);

        ctx.spawn(
            sink.success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", apply_req, e)),
        )
    }

    fn bulk_delete(
        &mut self,
        ctx: RpcContext,
        req: BulkDeleteReq,
        sink: UnarySink<BulkDeleteResp>,
    ) {
        debug!("request: {:?}", req);

        let mut apply_req = ApplyReq::new();
        apply_req.set_client_id(req.get_client_id());
        apply_req.set_req_type(ReqType::BulkDelete);
        apply_req.set_bulk_delete_req(req.to_owned());

        let (err, ret) = Self::apply(self, &apply_req);
        let mut resp = BulkDeleteResp::new();
        resp.set_err(err);
        resp.set_value(ret);

        debug!("response: {:?}", resp);

        ctx.spawn(
            sink.success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", apply_req, e)),
        )
    }

    fn commit(&mut self, ctx: RpcContext, req: CommitReq, sink: UnarySink<CommitResp>) {
        debug!("request: {:?}", req);

        let mut apply_req = ApplyReq::new();
        apply_req.set_client_id(req.get_client_id());
        apply_req.set_req_type(ReqType::Commit);
        apply_req.set_commit_req(req.to_owned());

        let (err, ret) = Self::apply(self, &apply_req);
        let mut resp = CommitResp::new();
        resp.set_err(err);
        resp.set_value(ret);

        debug!("response: {:?}", resp);

        ctx.spawn(
            sink.success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", apply_req, e)),
        )
    }

    fn rollback(&mut self, ctx: RpcContext, req: RollbackReq, sink: UnarySink<RollbackResp>) {
        debug!("request: {:?}", req);

        let mut apply_req = ApplyReq::new();
        apply_req.set_client_id(req.get_client_id());
        apply_req.set_req_type(ReqType::Rollback);
        apply_req.set_rollback_req(req.to_owned());

        let (err, ret) = Self::apply(self, &apply_req);
        let mut resp = RollbackResp::new();
        resp.set_err(err);
        resp.set_value(ret);

        debug!("response: {:?}", resp);

        ctx.spawn(
            sink.success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", apply_req, e)),
        )
    }

    fn merge(&mut self, ctx: RpcContext, req: MergeReq, sink: UnarySink<MergeResp>) {
        debug!("request: {:?}", req);

        let mut apply_req = ApplyReq::new();
        apply_req.set_client_id(req.get_client_id());
        apply_req.set_req_type(ReqType::Merge);
        apply_req.set_merge_req(req.to_owned());

        let (err, ret) = Self::apply(self, &apply_req);
        let mut resp = MergeResp::new();
        resp.set_err(err);
        resp.set_value(ret);

        debug!("response: {:?}", resp);

        ctx.spawn(
            sink.success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", apply_req, e)),
        )
    }

    fn search(&mut self, ctx: RpcContext, req: SearchReq, sink: UnarySink<SearchResp>) {
        debug!("request: {:?}", req);

        self.metrics.lock().unwrap().inc_request_count("search");

        let schema = self.index.schema();
        let default_fields: Vec<Field> = schema
            .fields()
            .flat_map(|(field, field_entry)| {
                if let FieldType::Str(text_field_options) = field_entry.field_type() {
                    if text_field_options.get_indexing_options().is_some() {
                        return Some(field);
                    }
                }
                None
            })
            .collect();

        let limit = req.get_from() + req.get_limit();

        let query_parser = QueryParser::for_index(&self.index, default_fields);
        let query = query_parser.parse_query(req.query.as_str()).unwrap();
        let searcher = self.index.reader().unwrap().searcher();
        let mut multi_collector = MultiCollector::new();
        let count_handle = if req.get_exclude_count() {
            None
        } else {
            Some(multi_collector.add_collector(Count))
        };
        let top_docs_handle = if req.get_exclude_docs() {
            None
        } else {
            Some(multi_collector.add_collector(TopDocs::with_limit(limit as usize)))
        };
        let facet_handle = if req.get_facet_field().is_empty() {
            None
        } else {
            let mut facet_collector =
                FacetCollector::for_field(schema.get_field(req.get_facet_field()).unwrap());
            for facet_prefix in req.get_facet_prefixes() {
                facet_collector.add_facet(facet_prefix);
            }
            Some(multi_collector.add_collector(facet_collector))
        };

        // search index
        let mut multi_fruit = searcher.search(&query, &multi_collector).unwrap();

        // count
        let mut count: i64 = -1;
        if let Some(ch) = count_handle {
            count = ch.extract(&mut multi_fruit) as i64;
        }

        // docs
        let mut top_docs = Vec::new();
        if let Some(tdh) = top_docs_handle {
            top_docs = tdh.extract(&mut multi_fruit);
        }

        // facet
        let mut facet: HashMap<String, HashMap<String, u64>> = HashMap::new();
        if let Some(fh) = facet_handle {
            let facet_counts = fh.extract(&mut multi_fruit);
            let mut facet_kv: HashMap<String, u64> = HashMap::new();
            for facet_prefix in req.get_facet_prefixes() {
                for (facet_key, facet_value) in facet_counts.get(facet_prefix) {
                    debug!("{:?}={}", facet_key.to_string(), facet_value);
                    facet_kv.insert(facet_key.to_string(), facet_value);
                }
            }
            facet.insert(req.get_facet_field().to_string(), facet_kv);
        }

        let mut docs: Vec<ScoredNamedFieldDocument> = Vec::new();
        let mut doc_pos: u64 = 0;
        for (score, doc_address) in top_docs {
            if doc_pos >= req.get_from() {
                let doc = searcher.doc(doc_address).unwrap();
                let named_doc = schema.to_named_doc(&doc);
                debug!(
                    "score: {:?} doc: {:?}",
                    score,
                    serde_json::to_string(&named_doc).unwrap()
                );

                let scored_doc = ScoredNamedFieldDocument {
                    fields: named_doc,
                    score,
                };
                docs.push(scored_doc);
            }
            doc_pos += 1;
        }

        let sr = SearchResult { docs, count, facet };

        let mut resp = SearchResp::new();
        resp.set_err(RespErr::OK);
        resp.set_value(serde_json::to_string(&sr).unwrap());

        debug!("response: {:?}", resp);

        ctx.spawn(
            sink.success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", req, e)),
        )
    }

    fn schema(&mut self, ctx: RpcContext, req: SchemaReq, sink: UnarySink<SchemaResp>) {
        debug!("request: {:?}", req);

        self.metrics.lock().unwrap().inc_request_count("schema");

        let mut resp = SchemaResp::new();
        resp.set_err(RespErr::OK);
        resp.set_value(format!(
            "{}",
            serde_json::to_string(&self.index.schema()).unwrap()
        ));

        debug!("response: {:?}", resp);

        ctx.spawn(
            sink.success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", req, e)),
        );
    }
}
