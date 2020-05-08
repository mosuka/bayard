use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs, thread};

use async_std::task::block_on;
use bayard_proto::proto::commonpb::State;
use bayard_proto::proto::indexpb::{
    BulkDeleteReply, BulkDeleteReq, BulkSetReply, BulkSetReq, CommitReply, CommitReq, DeleteReply,
    DeleteReq, GetReply, GetReq, MergeReply, MergeReq, RollbackReply, RollbackReq, SchemaReply,
    SchemaReq, SearchReply, SearchReq, SetReply, SetReq, StatusReply, StatusReq,
};
use bayard_proto::proto::indexpb_grpc::IndexService;
use futures::Future;
use grpcio::{RpcContext, UnarySink};
use log::*;
use prometheus::{CounterVec, HistogramVec};
use raft::storage::MemStorage;
use serde::{Deserialize, Serialize};
use stringreader::StringReader;
use tantivy::collector::{Count, FacetCollector, MultiCollector, TopDocs};
use tantivy::merge_policy::LogMergePolicy;
use tantivy::query::{QueryParser, TermQuery};
use tantivy::schema::{Field, FieldType, IndexRecordOption, Schema};
use tantivy::{Document, Index, IndexWriter, Term};

use crate::index::search_result::{ScoredNamedFieldDocument, SearchResult};
use crate::raft::config;
use crate::raft::config::NodeAddress;
use crate::raft::server::RaftServer;
use crate::tokenizer::tokenizer_initializer::TokenizerInitializer;

lazy_static! {
    static ref REQUEST_COUNTER: CounterVec = register_counter_vec!(
        "bayard_requests_total",
        "Total number of requests.",
        &["func"]
    )
    .unwrap();
    static ref REQUEST_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "bayard_request_duration_seconds",
        "The request latencies in seconds.",
        &["func"]
    )
    .unwrap();
    static ref APPLY_COUNTER: CounterVec = register_counter_vec!(
        "bayard_applies_total",
        "Total number of applies.",
        &["func"]
    )
    .unwrap();
    static ref APPLY_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "bayard_apply_duration_seconds",
        "The apply latencies in seconds.",
        &["func"]
    )
    .unwrap();
}

#[derive(Clone)]
pub struct IndexServer {
    id: u64,
    index: Arc<Index>,
    index_writer: Arc<Mutex<IndexWriter>>,
    sender: Sender<config::Msg>,
    seq: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Op {
    Get {},
    Search {},
    Set { doc: String },
    Delete { id: String },
    BulkSet { docs: String },
    BulkDelete { docs: String },
    Commit {},
    Rollback {},
    Merge {},
    Schema {},
    Status {},
}

impl IndexServer {
    pub fn new(
        index_dir: String,
        schema_file: &str,
        tokenizer_file: &str,
        indexer_threads: usize,
        indexer_memory_size: usize,
        raft_storage: MemStorage,
        id: u64,
        node_address: NodeAddress,
        addresses: HashMap<u64, NodeAddress>,
    ) -> (IndexServer, RaftServer) {
        let index_path = Path::new(&index_dir);

        let schema_content = fs::read_to_string(schema_file).unwrap();
        let schema: Schema =
            serde_json::from_str(&schema_content).expect("error while reading json");

        let index = if index_path.exists() {
            Index::open_in_dir(index_path).unwrap()
        } else {
            fs::create_dir_all(&index_path).unwrap_or_default();
            Index::create_in_dir(index_path, schema).unwrap()
        };

        if tokenizer_file != "" {
            let mut tokenizer_initializer = TokenizerInitializer::new();
            let tokenizer_content = fs::read_to_string(tokenizer_file).unwrap();
            tokenizer_initializer.configure(index.tokenizers(), tokenizer_content.as_str());
        }

        let index_writer = if indexer_threads > 0 {
            index
                .writer_with_num_threads(indexer_threads, indexer_memory_size)
                .unwrap()
        } else {
            index.writer(indexer_memory_size).unwrap()
        };
        index_writer.set_merge_policy(Box::new(LogMergePolicy::default()));

        let (rs, rr) = mpsc::channel();
        let (apply_s, apply_r) = mpsc::channel();
        thread::spawn(move || {
            config::init_and_run(raft_storage, rr, apply_s, id, node_address, addresses);
        });

        let index_server = IndexServer {
            id,
            index: Arc::new(index),
            index_writer: Arc::new(Mutex::new(index_writer)),
            sender: rs.clone(),
            seq: 0,
        };
        let raft_server = RaftServer::new(rs);

        let index = index_server.index.clone();
        let index_writer = index_server.index_writer.clone();
        thread::spawn(move || {
            apply_daemon(apply_r, index, index_writer);
        });

        return (index_server, raft_server);
    }
}

impl IndexService for IndexServer {
    fn get(&mut self, ctx: RpcContext, req: GetReq, sink: UnarySink<GetReply>) {
        let (s1, r1) = mpsc::channel();
        let id = self.id.clone();
        let index = Arc::clone(&self.index);
        let sender = self.sender.clone();
        let op = Op::Get {};
        let seq = self.seq;
        self.seq += 1;

        debug!("get: req={:?}", req);
        REQUEST_COUNTER.with_label_values(&["get"]).inc();
        let timer = REQUEST_HISTOGRAM.with_label_values(&["get"]).start_timer();

        sender
            .send(config::Msg::Propose {
                seq,
                op,
                cb: Box::new(move |leader_id: i32, addresses: Vec<u8>| {
                    let t = Term::from_field_text(
                        index.schema().get_field("_id").unwrap(),
                        req.get_id(),
                    );
                    let tq = TermQuery::new(t, IndexRecordOption::Basic);
                    let searcher = index.reader().unwrap().searcher();

                    let (state, doc) = match searcher.search(&tq, &TopDocs::with_limit(10)) {
                        Ok(top_docs) => {
                            if top_docs.len() > 0 {
                                let mut doc = Document::default();
                                for (_score, doc_address) in top_docs {
                                    doc = searcher.doc(doc_address).unwrap();
                                }
                                let named_doc = index.schema().to_named_doc(&doc);
                                (State::OK, serde_json::to_string(&named_doc).unwrap())
                            } else {
                                (State::NOT_FOUND, "".to_string())
                            }
                        }
                        Err(_e) => (State::IO_ERROR, "".to_string()),
                    };

                    let mut reply = GetReply::new();
                    reply.set_state(state);
                    if leader_id >= 0 {
                        // not a leader
                        reply.set_leader_id(leader_id as u64);
                    } else {
                        // a leader
                        reply.set_leader_id(id);
                    }
                    reply.set_doc(doc);
                    reply.set_address_map(addresses);
                    s1.send(reply).expect("cb channel closed");
                }),
            })
            .unwrap();

        let reply = match r1.recv_timeout(Duration::from_secs(2)) {
            Ok(r) => r,
            Err(_e) => {
                let mut r = GetReply::new();
                r.set_state(State::IO_ERROR);
                r
            }
        };

        let f = sink
            .success(reply.clone())
            .map_err(move |err| error!("Failed to reply get: {:?}", err));
        ctx.spawn(f);

        timer.observe_duration();
    }

    fn search(&mut self, ctx: RpcContext, req: SearchReq, sink: UnarySink<SearchReply>) {
        let (s1, r1) = mpsc::channel();
        let id = self.id.clone();
        let index = Arc::clone(&self.index);
        let sender = self.sender.clone();
        let op = Op::Search {};
        let seq = self.seq;
        self.seq += 1;

        debug!("search: req={:?}", req);
        REQUEST_COUNTER.with_label_values(&["search"]).inc();
        let timer = REQUEST_HISTOGRAM
            .with_label_values(&["search"])
            .start_timer();

        sender
            .send(config::Msg::Propose {
                seq,
                op,
                cb: Box::new(move |leader_id: i32, addresses: Vec<u8>| {
                    let default_fields: Vec<Field> = index
                        .schema()
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
                    let query_parser = QueryParser::for_index(&index, default_fields);
                    let query = query_parser.parse_query(req.query.as_str()).unwrap();
                    let searcher = index.reader().unwrap().searcher();
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
                        let mut facet_collector = FacetCollector::for_field(
                            index.schema().get_field(req.get_facet_field()).unwrap(),
                        );
                        for facet_prefix in req.get_facet_prefixes() {
                            facet_collector.add_facet(facet_prefix);
                        }
                        Some(multi_collector.add_collector(facet_collector))
                    };

                    // search index
                    let (state, search_result) = match searcher.search(&query, &multi_collector) {
                        Ok(mut multi_fruit) => {
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
                                        facet_kv.insert(facet_key.to_string(), facet_value);
                                    }
                                }
                                facet.insert(req.get_facet_field().to_string(), facet_kv);
                            }

                            // docs
                            let mut docs: Vec<ScoredNamedFieldDocument> = Vec::new();
                            let mut doc_pos: u64 = 0;
                            for (score, doc_address) in top_docs {
                                if doc_pos >= req.get_from() {
                                    let doc = searcher.doc(doc_address).unwrap();
                                    let named_doc = index.schema().to_named_doc(&doc);
                                    let scored_doc = ScoredNamedFieldDocument {
                                        fields: named_doc,
                                        score,
                                    };
                                    docs.push(scored_doc);
                                }
                                doc_pos += 1;
                            }

                            // search result
                            let search_result = SearchResult { docs, count, facet };

                            (State::OK, serde_json::to_string(&search_result).unwrap())
                        }
                        Err(_e) => (State::IO_ERROR, "".to_string()),
                    };

                    let mut reply = SearchReply::new();
                    reply.set_state(state);
                    if leader_id >= 0 {
                        // not a leader
                        reply.set_leader_id(leader_id as u64);
                    } else {
                        // a leader
                        reply.set_leader_id(id);
                    }
                    reply.set_search_result(search_result);
                    reply.set_address_map(addresses);
                    s1.send(reply).expect("cb channel closed");
                }),
            })
            .unwrap();

        let reply = match r1.recv_timeout(Duration::from_secs(2)) {
            Ok(r) => r,
            Err(_e) => {
                let mut r = SearchReply::new();
                r.set_state(State::IO_ERROR);
                r
            }
        };

        let f = sink
            .success(reply.clone())
            .map_err(move |err| error!("Failed to reply search: {:?}", err));
        ctx.spawn(f);

        timer.observe_duration();
    }

    fn set(&mut self, ctx: RpcContext, req: SetReq, sink: UnarySink<SetReply>) {
        let (s1, r1) = mpsc::channel();
        let id = self.id.clone();
        let sender = self.sender.clone();
        let op = Op::Set {
            doc: String::from(req.get_doc()),
        };
        let seq = self.seq;
        self.seq += 1;

        debug!("set: req={:?}", req);
        REQUEST_COUNTER.with_label_values(&["set"]).inc();
        let timer = REQUEST_HISTOGRAM.with_label_values(&["set"]).start_timer();

        sender
            .send(config::Msg::Propose {
                seq,
                op,
                cb: Box::new(move |leader_id: i32, addresses: Vec<u8>| {
                    let mut reply = SetReply::new();
                    if leader_id >= 0 {
                        // not a leader
                        reply.set_state(State::WRONG_LEADER);
                        reply.set_leader_id(leader_id as u64);
                    } else {
                        // a leader
                        reply.set_state(State::OK);
                        reply.set_leader_id(id);
                    }
                    reply.set_address_map(addresses);
                    // done job, wake
                    s1.send(reply).expect("cb channel closed");
                }),
            })
            .unwrap();

        let reply = match r1.recv_timeout(Duration::from_secs(2)) {
            Ok(r) => r,
            Err(_e) => {
                let mut r = SetReply::new();
                r.set_state(State::IO_ERROR);
                r
            }
        };

        let f = sink
            .success(reply.clone())
            .map_err(move |err| error!("Failed to reply set: {:?}", err));
        ctx.spawn(f);

        timer.observe_duration();
    }

    fn delete(&mut self, ctx: RpcContext, req: DeleteReq, sink: UnarySink<DeleteReply>) {
        let (s1, r1) = mpsc::channel();
        let id = self.id.clone();
        let sender = self.sender.clone();
        let op = Op::Delete {
            id: String::from(req.get_id()),
        };
        let seq = self.seq;
        self.seq += 1;

        debug!("delete: req={:?}", req);
        REQUEST_COUNTER.with_label_values(&["delete"]).inc();
        let timer = REQUEST_HISTOGRAM
            .with_label_values(&["delete"])
            .start_timer();

        sender
            .send(config::Msg::Propose {
                seq,
                op,
                cb: Box::new(move |leader_id: i32, addresses: Vec<u8>| {
                    let mut reply = DeleteReply::new();
                    if leader_id >= 0 {
                        // not a leader
                        reply.set_state(State::WRONG_LEADER);
                        reply.set_leader_id(leader_id as u64);
                    } else {
                        // a leader
                        reply.set_state(State::OK);
                        reply.set_leader_id(id);
                    }
                    reply.set_address_map(addresses);
                    // done job, wake
                    s1.send(reply).expect("cb channel closed");
                }),
            })
            .unwrap();

        let reply = match r1.recv_timeout(Duration::from_secs(2)) {
            Ok(r) => r,
            Err(_e) => {
                let mut r = DeleteReply::new();
                r.set_state(State::IO_ERROR);
                r
            }
        };

        let f = sink
            .success(reply.clone())
            .map_err(move |err| error!("Failed to reply delete: {:?}", err));
        ctx.spawn(f);

        timer.observe_duration();
    }

    fn bulk_set(&mut self, ctx: RpcContext, req: BulkSetReq, sink: UnarySink<BulkSetReply>) {
        let (s1, r1) = mpsc::channel();
        let id = self.id.clone();
        let sender = self.sender.clone();
        let op = Op::BulkSet {
            docs: String::from(req.get_docs()),
        };
        let seq = self.seq;
        self.seq += 1;

        debug!("bulk_set");
        REQUEST_COUNTER.with_label_values(&["bulk_set"]).inc();
        let timer = REQUEST_HISTOGRAM
            .with_label_values(&["bulk_set"])
            .start_timer();

        sender
            .send(config::Msg::Propose {
                seq,
                op,
                cb: Box::new(move |leader_id: i32, addresses: Vec<u8>| {
                    let mut reply = BulkSetReply::new();
                    if leader_id >= 0 {
                        // not a leader
                        reply.set_state(State::WRONG_LEADER);
                        reply.set_leader_id(leader_id as u64);
                    } else {
                        // a leader
                        reply.set_state(State::OK);
                        reply.set_leader_id(id);
                    }
                    reply.set_address_map(addresses);
                    // done job, wake
                    s1.send(reply).expect("cb channel closed");
                }),
            })
            .unwrap();

        let reply = match r1.recv_timeout(Duration::from_secs(2)) {
            Ok(r) => r,
            Err(_e) => {
                let mut r = BulkSetReply::new();
                r.set_state(State::IO_ERROR);
                r
            }
        };

        let f = sink
            .success(reply.clone())
            .map_err(move |err| error!("Failed to reply bulk set: {:?}", err));
        ctx.spawn(f);

        timer.observe_duration();
    }

    fn bulk_delete(
        &mut self,
        ctx: RpcContext,
        req: BulkDeleteReq,
        sink: UnarySink<BulkDeleteReply>,
    ) {
        let (s1, r1) = mpsc::channel();
        let id = self.id.clone();
        let sender = self.sender.clone();
        let op = Op::BulkDelete {
            docs: String::from(req.get_docs()),
        };
        let seq = self.seq;
        self.seq += 1;

        debug!("bulk_delete");
        REQUEST_COUNTER.with_label_values(&["bulk_delete"]).inc();
        let timer = REQUEST_HISTOGRAM
            .with_label_values(&["bulk_delete"])
            .start_timer();

        sender
            .send(config::Msg::Propose {
                seq,
                op,
                cb: Box::new(move |leader_id: i32, addresses: Vec<u8>| {
                    let mut reply = BulkDeleteReply::new();
                    if leader_id >= 0 {
                        // not a leader
                        reply.set_state(State::WRONG_LEADER);
                        reply.set_leader_id(leader_id as u64);
                    } else {
                        // a leader
                        reply.set_state(State::OK);
                        reply.set_leader_id(id);
                    }
                    reply.set_address_map(addresses);
                    // done job, wake
                    s1.send(reply).expect("cb channel closed");
                }),
            })
            .unwrap();

        let reply = match r1.recv_timeout(Duration::from_secs(2)) {
            Ok(r) => r,
            Err(_e) => {
                let mut r = BulkDeleteReply::new();
                r.set_state(State::IO_ERROR);
                r
            }
        };

        let f = sink
            .success(reply.clone())
            .map_err(move |err| error!("Failed to reply bulk delete: {:?}", err));
        ctx.spawn(f);

        timer.observe_duration();
    }

    fn commit(&mut self, ctx: RpcContext, req: CommitReq, sink: UnarySink<CommitReply>) {
        let (s1, r1) = mpsc::channel();
        let id = self.id.clone();
        let sender = self.sender.clone();
        let op = Op::Commit {};
        let seq = self.seq;
        self.seq += 1;

        debug!("commit: req={:?}", req);
        REQUEST_COUNTER.with_label_values(&["commit"]).inc();
        let timer = REQUEST_HISTOGRAM
            .with_label_values(&["commit"])
            .start_timer();

        sender
            .send(config::Msg::Propose {
                seq,
                op,
                cb: Box::new(move |leader_id: i32, addresses: Vec<u8>| {
                    let mut reply = CommitReply::new();
                    if leader_id >= 0 {
                        // not a leader
                        reply.set_state(State::WRONG_LEADER);
                        reply.set_leader_id(leader_id as u64);
                    } else {
                        // a leader
                        reply.set_state(State::OK);
                        reply.set_leader_id(id);
                    }
                    reply.set_address_map(addresses);
                    // done job, wake
                    s1.send(reply).expect("cb channel closed");
                }),
            })
            .unwrap();

        let reply = match r1.recv_timeout(Duration::from_secs(2)) {
            Ok(r) => r,
            Err(_e) => {
                let mut r = CommitReply::new();
                r.set_state(State::IO_ERROR);
                r
            }
        };

        let f = sink
            .success(reply.clone())
            .map_err(move |err| error!("Failed to reply commit: {:?}", err));
        ctx.spawn(f);

        timer.observe_duration();
    }

    fn rollback(&mut self, ctx: RpcContext, req: RollbackReq, sink: UnarySink<RollbackReply>) {
        let (s1, r1) = mpsc::channel();
        let id = self.id.clone();
        let sender = self.sender.clone();
        let op = Op::Rollback {};
        let seq = self.seq;
        self.seq += 1;

        debug!("rollback: req={:?}", req);
        REQUEST_COUNTER.with_label_values(&["rollback"]).inc();
        let timer = REQUEST_HISTOGRAM
            .with_label_values(&["rollback"])
            .start_timer();

        sender
            .send(config::Msg::Propose {
                seq,
                op,
                cb: Box::new(move |leader_id: i32, addresses: Vec<u8>| {
                    let mut reply = RollbackReply::new();
                    if leader_id >= 0 {
                        // not a leader
                        reply.set_state(State::WRONG_LEADER);
                        reply.set_leader_id(leader_id as u64);
                    } else {
                        // a leader
                        reply.set_state(State::OK);
                        reply.set_leader_id(id);
                    }
                    reply.set_address_map(addresses);
                    // done job, wake
                    s1.send(reply).expect("cb channel closed");
                }),
            })
            .unwrap();

        let reply = match r1.recv_timeout(Duration::from_secs(2)) {
            Ok(r) => r,
            Err(_e) => {
                let mut r = RollbackReply::new();
                r.set_state(State::IO_ERROR);
                r
            }
        };

        let f = sink
            .success(reply.clone())
            .map_err(move |err| error!("Failed to reply commit: {:?}", err));
        ctx.spawn(f);

        timer.observe_duration();
    }

    fn merge(&mut self, ctx: RpcContext, req: MergeReq, sink: UnarySink<MergeReply>) {
        let (s1, r1) = mpsc::channel();
        let id = self.id.clone();
        let sender = self.sender.clone();
        let op = Op::Merge {};
        let seq = self.seq;
        self.seq += 1;

        debug!("merge: req={:?}", req);
        REQUEST_COUNTER.with_label_values(&["merge"]).inc();
        let timer = REQUEST_HISTOGRAM
            .with_label_values(&["merge"])
            .start_timer();

        sender
            .send(config::Msg::Propose {
                seq,
                op,
                cb: Box::new(move |leader_id: i32, addresses: Vec<u8>| {
                    let mut reply = MergeReply::new();
                    if leader_id >= 0 {
                        // not a leader
                        reply.set_state(State::WRONG_LEADER);
                        reply.set_leader_id(leader_id as u64);
                    } else {
                        // a leader
                        reply.set_state(State::OK);
                        reply.set_leader_id(id);
                    }
                    reply.set_address_map(addresses);
                    // done job, wake
                    s1.send(reply).expect("cb channel closed");
                }),
            })
            .unwrap();

        let reply = match r1.recv_timeout(Duration::from_secs(2)) {
            Ok(r) => r,
            Err(_e) => {
                let mut r = MergeReply::new();
                r.set_state(State::IO_ERROR);
                r
            }
        };

        let f = sink
            .success(reply.clone())
            .map_err(move |err| error!("Failed to reply commit: {:?}", err));
        ctx.spawn(f);

        timer.observe_duration();
    }

    fn schema(&mut self, ctx: RpcContext, req: SchemaReq, sink: UnarySink<SchemaReply>) {
        let (s1, r1) = mpsc::channel();
        let id = self.id.clone();
        let index = Arc::clone(&self.index);
        let sender = self.sender.clone();
        let op = Op::Schema {};
        let seq = self.seq;
        self.seq += 1;

        debug!("schema: req={:?}", req);
        REQUEST_COUNTER.with_label_values(&["schema"]).inc();
        let timer = REQUEST_HISTOGRAM
            .with_label_values(&["schema"])
            .start_timer();

        sender
            .send(config::Msg::Propose {
                seq,
                op,
                cb: Box::new(move |leader_id: i32, addresses: Vec<u8>| {
                    let (state, schema) = match serde_json::to_string(&index.schema()) {
                        Ok(s) => (State::OK, s),
                        _ => (State::IO_ERROR, "".to_string()),
                    };

                    let mut reply = SchemaReply::new();
                    reply.set_state(state);
                    if leader_id >= 0 {
                        // not a leader
                        reply.set_leader_id(leader_id as u64);
                    } else {
                        // a leader
                        reply.set_leader_id(id);
                    }
                    reply.set_schema(schema);
                    reply.set_address_map(addresses);
                    s1.send(reply).expect("cb channel closed");
                }),
            })
            .unwrap();

        let reply = match r1.recv_timeout(Duration::from_secs(2)) {
            Ok(r) => r,
            Err(_e) => {
                let mut r = SchemaReply::new();
                r.set_state(State::IO_ERROR);
                r
            }
        };

        let f = sink
            .success(reply.clone())
            .map_err(move |err| error!("Failed to reply schema: {:?}", err));
        ctx.spawn(f);

        timer.observe_duration();
    }

    fn status(&mut self, ctx: RpcContext, req: StatusReq, sink: UnarySink<StatusReply>) {
        let (s1, r1) = mpsc::channel();
        let id = self.id.clone();
        let _index = Arc::clone(&self.index);
        let sender = self.sender.clone();
        let op = Op::Status {};
        let seq = self.seq;
        self.seq += 1;

        debug!("status: req={:?}", req);
        REQUEST_COUNTER.with_label_values(&["status"]).inc();
        let timer = REQUEST_HISTOGRAM
            .with_label_values(&["status"])
            .start_timer();

        sender
            .send(config::Msg::Propose {
                seq,
                op,
                cb: Box::new(move |leader_id: i32, addresses: Vec<u8>| {
                    // TODO: check node status. for example, index, peers and etc.
                    let status = "OK";

                    let (state, status) = (State::OK, status.to_string());

                    let mut reply = StatusReply::new();
                    reply.set_state(state);
                    if leader_id >= 0 {
                        // not a leader
                        reply.set_leader_id(leader_id as u64);
                    } else {
                        // a leader
                        reply.set_leader_id(id);
                    }
                    reply.set_status(status);
                    reply.set_address_map(addresses);
                    s1.send(reply).expect("cb channel closed");
                }),
            })
            .unwrap();

        let reply = match r1.recv_timeout(Duration::from_secs(2)) {
            Ok(r) => r,
            Err(_e) => {
                let mut r = StatusReply::new();
                r.set_state(State::IO_ERROR);
                r
            }
        };

        let f = sink
            .success(reply.clone())
            .map_err(move |err| error!("Failed to reply status: {:?}", err));
        ctx.spawn(f);

        timer.observe_duration();
    }
}

fn apply_daemon(receiver: Receiver<Op>, index: Arc<Index>, index_writer: Arc<Mutex<IndexWriter>>) {
    loop {
        let op = match receiver.recv() {
            Ok(o) => o,
            _ => {
                debug!("apply daemon return");
                return;
            }
        };
        match op {
            Op::Get {} => {
                // noop
            }
            Op::Search {} => {
                // noop
            }
            Op::Set { doc } => {
                APPLY_COUNTER.with_label_values(&["set"]).inc();
                let timer = APPLY_HISTOGRAM.with_label_values(&["set"]).start_timer();

                let id_field = index.schema().get_field("_id").unwrap();

                let doc = index.schema().parse_document(doc.as_str()).unwrap();
                let id = &doc.get_first(id_field).unwrap().text().unwrap().to_string();

                let opstamp = index_writer
                    .lock()
                    .unwrap()
                    .delete_term(Term::from_field_text(id_field, id));
                info!("delete document: id={:?}, opstamp={}", id, opstamp);

                let opstamp = index_writer.lock().unwrap().add_document(doc);
                info!("add document: id={:?}, opstamp={}", id, opstamp);

                timer.observe_duration();
            }
            Op::Delete { id } => {
                APPLY_COUNTER.with_label_values(&["delete"]).inc();
                let timer = APPLY_HISTOGRAM.with_label_values(&["delete"]).start_timer();

                let id_field = index.schema().get_field("_id").unwrap();

                let opstamp = index_writer
                    .lock()
                    .unwrap()
                    .delete_term(Term::from_field_text(id_field, id.as_str()));
                info!("delete document: id={}, opstamp={}", id, opstamp);

                timer.observe_duration();
            }
            Op::BulkSet { docs } => {
                APPLY_COUNTER.with_label_values(&["bulk_set"]).inc();
                let timer = APPLY_HISTOGRAM
                    .with_label_values(&["bulk_set"])
                    .start_timer();

                let id_field = index.schema().get_field("_id").unwrap();

                let mut cnt = 0;

                let mut reader = BufReader::new(StringReader::new(docs.as_str()));
                let mut line = String::new();
                while reader.read_line(&mut line).unwrap() > 0 {
                    let doc = index.schema().parse_document(&line).unwrap();
                    let id = &doc.get_first(id_field).unwrap().text().unwrap().to_string();

                    let opstamp = index_writer
                        .lock()
                        .unwrap()
                        .delete_term(Term::from_field_text(id_field, id));
                    info!("delete document: id={:?}, opstamp={}", id, opstamp);

                    let opstamp = index_writer.lock().unwrap().add_document(doc);
                    info!("add document: id={:?}, opstamp={}", id, opstamp);

                    line.clear();

                    cnt += 1;
                }

                info!("set {} documents", cnt);

                timer.observe_duration();
            }
            Op::BulkDelete { docs } => {
                APPLY_COUNTER.with_label_values(&["bulk_delete"]).inc();
                let timer = APPLY_HISTOGRAM
                    .with_label_values(&["bulk_delete"])
                    .start_timer();

                let id_field = index.schema().get_field("_id").unwrap();

                let mut cnt = 0;

                let mut reader = BufReader::new(StringReader::new(docs.as_str()));
                let mut line = String::new();
                while reader.read_line(&mut line).unwrap() > 0 {
                    let doc = index.schema().parse_document(&line).unwrap();
                    let id = &doc.get_first(id_field).unwrap().text().unwrap().to_string();

                    let opstamp = index_writer
                        .lock()
                        .unwrap()
                        .delete_term(Term::from_field_text(id_field, id));
                    info!("delete document: id={:?}, opstamp={}", id, opstamp);

                    line.clear();

                    cnt += 1;
                }

                info!("delete {} documents", cnt);

                timer.observe_duration();
            }
            Op::Commit {} => {
                APPLY_COUNTER.with_label_values(&["commit"]).inc();
                let timer = APPLY_HISTOGRAM.with_label_values(&["commit"]).start_timer();

                match index_writer.lock().unwrap().commit() {
                    Ok(opstamp) => {
                        info!("commit index: opstamp={}", opstamp);
                    }
                    Err(e) => {
                        error!("failed to commit index: {}", e);
                    }
                };

                timer.observe_duration();
            }
            Op::Rollback {} => {
                APPLY_COUNTER.with_label_values(&["rollback"]).inc();
                let timer = APPLY_HISTOGRAM
                    .with_label_values(&["rollback"])
                    .start_timer();

                match index_writer.lock().unwrap().rollback() {
                    Ok(opstamp) => {
                        info!("rollback index: opstamp={}", opstamp);
                    }
                    Err(e) => {
                        error!("failed to rollback index: {}", e);
                    }
                };

                timer.observe_duration();
            }
            Op::Merge {} => {
                APPLY_COUNTER.with_label_values(&["rollback"]).inc();
                let timer = APPLY_HISTOGRAM
                    .with_label_values(&["rollback"])
                    .start_timer();

                let segments = index.searchable_segment_ids().unwrap();
                if segments.len() <= 0 {
                    info!("there are no segment files that can be merged");
                    return;
                }

                let merge_future = index_writer.lock().unwrap().merge(&segments);
                match block_on(merge_future) {
                    Ok(segment_meta) => {
                        info!("merge index: segments={:?}", segment_meta);
                    }
                    Err(e) => {
                        error!("failed to merge index: {:?}", e);
                    }
                };

                timer.observe_duration();
            }
            Op::Schema {} => {
                // noop
            }
            Op::Status {} => {
                // noop
            }
        }
    }
}
