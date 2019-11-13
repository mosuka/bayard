use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crossbeam_channel::{bounded, select, Receiver as CReceiver};
use ctrlc;
use futures::Future;
use grpcio::{ChannelBuilder, EnvBuilder, Environment, RpcContext, ServerBuilder, UnarySink};
use log::*;
use protobuf::Message;
use raft::eraftpb::{ConfChange, Entry, EntryType, Message as RaftMessage};
use tantivy::collector::TopDocs;
use tantivy::query::{QueryParser, TermQuery};
use tantivy::schema::{Field, FieldType, IndexRecordOption, NamedFieldDocument};
use tantivy::{Document, Index, Term};

use crate::proto::indexpb_grpc::{self, Index as IndexService, IndexClient};
use crate::proto::indexrpcpb::{
    ConfChangeReq, DeleteResp, GetResp, IndexReq, PutResp, RaftDone, ReqType, RespErr, SearchResp,
};
use crate::server::peer::PeerMessage;
use crate::server::{peer, util};

struct NotifyArgs(u64, String, RespErr);

fn sigterm_channel() -> Result<CReceiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })
    .unwrap();

    Ok(receiver)
}

#[derive(Clone)]
pub struct IndexServer {
    id: u64,
    peers: Arc<Mutex<HashMap<u64, IndexClient>>>,
    rf_message_ch: SyncSender<PeerMessage>,
    notify_ch_map: Arc<Mutex<HashMap<u64, SyncSender<NotifyArgs>>>>,
    index: Arc<Index>,
    unique_key_field: String,
}

impl IndexServer {
    pub fn start_server(
        id: u64,
        host: &str,
        port: u16,
        peers: HashMap<u64, IndexClient>,
        index: Arc<Index>,
        unique_key_field: &str,
    ) {
        let (rf_sender, rf_receiver) = mpsc::sync_channel(100);
        let (rpc_sender, rpc_receiver) = mpsc::sync_channel(100);
        let (apply_sender, apply_receiver) = mpsc::sync_channel(100);

        let peers_id = peers.keys().map(|id| *id).collect();
        let peer = peer::Peer::new(id, apply_sender, peers_id);

        let mut index_server = IndexServer {
            id,
            peers: Arc::new(Mutex::new(peers)),
            rf_message_ch: rf_sender,
            notify_ch_map: Arc::new(Mutex::new(HashMap::new())),
            index,
            unique_key_field: unique_key_field.to_string(),
        };

        index_server.async_rpc_sender(rpc_receiver);
        index_server.async_applier(apply_receiver);

        let env = Arc::new(Environment::new(10));
        let service = indexpb_grpc::create_index(index_server);
        let mut server = ServerBuilder::new(env)
            .register_service(service)
            .bind(host, port)
            .build()
            .unwrap_or_else(|e| {
                panic!("build server error: {}", e);
            });

        peer::Peer::activate(peer, rpc_sender, rf_receiver);
        server.start();
        for &(ref host, port) in server.bind_addrs() {
            info!("listening on {}:{}", host, port);
        }

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

    fn start_op(&mut self, req: &IndexReq) -> (RespErr, String) {
        let (sh, rh) = mpsc::sync_channel(0);
        {
            let mut map = self.notify_ch_map.lock().unwrap();
            map.insert(req.get_client_id(), sh);
        }
        self.rf_message_ch
            .send(PeerMessage::Propose(req.write_to_bytes().unwrap_or_else(
                |e| {
                    panic!("request write to bytes error: {}", e);
                },
            )))
            .unwrap_or_else(|e| {
                error!("send propose to raft error: {}", e);
            });
        match rh.recv_timeout(Duration::from_millis(1000)) {
            Ok(args) => {
                return (args.2, args.1);
            }
            Err(_) => {
                {
                    let mut map = self.notify_ch_map.lock().unwrap();
                    map.remove(&req.get_client_id());
                }
                return (RespErr::ErrWrongLeader, String::from(""));
            }
        }
    }

    // TODO: check duplicate request.
    fn async_applier(&mut self, apply_receiver: Receiver<Entry>) {
        let notify_ch_map = self.notify_ch_map.clone();
        let peers = self.peers.clone();
        let index = self.index.clone();
        let unique_key_field = self.unique_key_field.clone();

        thread::spawn(move || loop {
            match apply_receiver.recv() {
                Ok(e) => match e.get_entry_type() {
                    EntryType::EntryNormal => {
                        let result: NotifyArgs;
                        let req: IndexReq = util::parse_data(e.get_data());
                        let client_id = req.get_client_id();
                        if e.data.len() > 0 {
                            result = Self::apply_entry(
                                e.term,
                                &req,
                                peers.clone(),
                                index.clone(),
                                unique_key_field.as_str(),
                            );
                            debug!("apply_entry: {:?}---{:?}", req, result.2);
                        } else {
                            result = NotifyArgs(0, String::from(""), RespErr::ErrWrongLeader);
                            debug!("empty_entry: {:?}", req);
                        }
                        let mut map = notify_ch_map.lock().unwrap();
                        if let Some(s) = map.get(&client_id) {
                            s.send(result).unwrap_or_else(|e| {
                                error!("notify apply result error: {}", e);
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
                                error!("notify apply result error: {}", e);
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
        req: &IndexReq,
        peers: Arc<Mutex<HashMap<u64, IndexClient>>>,
        index: Arc<Index>,
        unique_key_field: &str,
    ) -> NotifyArgs {
        match req.req_type {
            ReqType::Get => {
                // searcher
                debug!("Get");
                let t = Term::from_field_text(
                    index.schema().get_field(unique_key_field).unwrap(),
                    req.key.as_str(),
                );
                let tq = TermQuery::new(t, IndexRecordOption::Basic);
                let searcher = index.reader().unwrap().searcher();
                let top_docs = searcher.search(&tq, &TopDocs::with_limit(10)).unwrap();
                let mut doc = Document::default();
                for (_score, doc_address) in top_docs {
                    doc = searcher.doc(doc_address).unwrap();
                }
                let named_doc = index.schema().to_named_doc(&doc);
                NotifyArgs(
                    term,
                    serde_json::to_string(&named_doc).unwrap(),
                    RespErr::OK,
                )
            }
            ReqType::Put => {
                // indexer
                debug!("Put");
                let num_threads = 1;
                let buffer_size_per_thread = 50_000_000;
                let mut index_writer = if num_threads > 0 {
                    index
                        .writer_with_num_threads(num_threads, buffer_size_per_thread)
                        .unwrap()
                } else {
                    index.writer(buffer_size_per_thread).unwrap()
                };
                let mut doc = index.schema().parse_document(req.value.as_str()).unwrap();
                let field = index.schema().get_field(unique_key_field).unwrap();
                doc.add_text(field, req.key.as_str());
                index_writer.delete_term(Term::from_field_text(field, req.key.as_str()));
                index_writer.add_document(doc);
                match index_writer.commit() {
                    Ok(opstamp) => {
                        debug!("Commit succeed, opstamp at {}", opstamp);
                        NotifyArgs(term, String::from(""), RespErr::OK)
                    }
                    Err(e) => {
                        error!("index error: {}", e);
                        NotifyArgs(term, String::from(""), RespErr::ErrWrongLeader)
                    }
                }
            }
            ReqType::Delete => {
                // indexer
                debug!("Delete");
                let num_threads = 1;
                let buffer_size_per_thread = 50_000_000;
                let mut index_writer = if num_threads > 0 {
                    index
                        .writer_with_num_threads(num_threads, buffer_size_per_thread)
                        .unwrap()
                } else {
                    index.writer(buffer_size_per_thread).unwrap()
                };
                index_writer.delete_term(Term::from_field_text(
                    index.schema().get_field(unique_key_field).unwrap(),
                    req.key.as_str(),
                ));
                match index_writer.commit() {
                    Ok(opstamp) => {
                        debug!("Commit succeed, opstamp at {}", opstamp);
                        NotifyArgs(term, String::from(""), RespErr::OK)
                    }
                    Err(e) => {
                        error!("index error: {}", e);
                        NotifyArgs(term, String::from(""), RespErr::ErrWrongLeader)
                    }
                }
            }
            ReqType::Search => {
                // searcher
                debug!("Search");
                let schema = index.schema();
                let default_fields: Vec<Field> = schema
                    .fields()
                    .iter()
                    .enumerate()
                    .filter(|&(_, ref field_entry)| match *field_entry.field_type() {
                        FieldType::Str(ref text_field_options) => {
                            text_field_options.get_indexing_options().is_some()
                        }
                        _ => false,
                    })
                    .map(|(i, _)| Field(i as u32))
                    .collect();
                let query_parser = QueryParser::for_index(&index, default_fields);
                let query = query_parser.parse_query(req.query.as_str()).unwrap();
                let searcher = index.reader().unwrap().searcher();
                let top_docs = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();
                let mut named_docs: Vec<NamedFieldDocument> = Vec::new();
                for (score, doc_address) in top_docs {
                    let doc = searcher.doc(doc_address).unwrap();
                    let named_doc = schema.to_named_doc(&doc);
                    debug!(
                        "score={} doc={}",
                        score,
                        serde_json::to_string(&named_doc).unwrap()
                    );
                    named_docs.push(named_doc);
                }
                NotifyArgs(
                    term,
                    serde_json::to_string(&named_docs).unwrap(),
                    RespErr::OK,
                )
            }
            ReqType::PeerAddr => {
                let mut prs = peers.lock().unwrap();
                let env = Arc::new(EnvBuilder::new().build());
                let ch = ChannelBuilder::new(env).connect(&req.peer_addr);
                prs.insert(req.peer_id, IndexClient::new(ch));
                NotifyArgs(term, String::from(""), RespErr::OK)
            }
        }
    }
}

impl IndexService for IndexServer {
    fn get(&mut self, ctx: RpcContext, req: IndexReq, sink: UnarySink<GetResp>) {
        let (err, value) = Self::start_op(self, &req);
        let mut resp = GetResp::new();
        resp.set_err(err);
        resp.set_value(value);
        ctx.spawn(
            sink.success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", req, e)),
        )
    }

    fn put(&mut self, ctx: RpcContext, req: IndexReq, sink: UnarySink<PutResp>) {
        let (err, _) = Self::start_op(self, &req);
        let mut resp = PutResp::new();
        resp.set_err(err);
        ctx.spawn(
            sink.success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", req, e)),
        )
    }

    fn delete(&mut self, ctx: RpcContext, req: IndexReq, sink: UnarySink<DeleteResp>) {
        let (err, _) = Self::start_op(self, &req);
        let mut resp = DeleteResp::new();
        resp.set_err(err);
        ctx.spawn(
            sink.success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", req, e)),
        )
    }

    fn search(&mut self, ctx: RpcContext, req: IndexReq, sink: UnarySink<SearchResp>) {
        let (err, value) = Self::start_op(self, &req);
        let mut resp = SearchResp::new();
        resp.set_err(err);
        resp.set_value(value);
        ctx.spawn(
            sink.success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", req, e)),
        )
    }

    fn raft(&mut self, ctx: RpcContext, req: RaftMessage, sink: UnarySink<RaftDone>) {
        self.rf_message_ch
            .send(PeerMessage::Message(req.clone()))
            .unwrap_or_else(|e| {
                error!("send message to raft error: {}", e);
            });
        let resp = RaftDone::new();
        ctx.spawn(
            sink.success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", req, e)),
        )
    }

    fn raft_conf_change(&mut self, ctx: RpcContext, req: ConfChangeReq, sink: UnarySink<RaftDone>) {
        let cc = req.cc.clone().unwrap();
        let mut resp = RaftDone::new();
        let mut peer_req = IndexReq::new();
        peer_req.set_req_type(ReqType::PeerAddr);
        peer_req.set_peer_addr(format!("{}:{}", req.ip, req.port));
        peer_req.set_peer_id(cc.get_node_id());
        peer_req.set_client_id(cc.get_node_id());
        let (err, _) = self.start_op(&peer_req);
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

        ctx.spawn(
            sink.success(resp)
                .map_err(move |e| error!("failed to reply {:?}: {:?}", req, e)),
        )
    }
}
