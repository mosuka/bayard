use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use grpcio::{ChannelBuilder, EnvBuilder};
use log::*;
use protobuf::RepeatedField;
use raft::eraftpb::{ConfChange, ConfChangeType};

use crate::proto::indexpb_grpc::IndexClient;
use crate::proto::indexrpcpb::{
    ApplyReq, BulkDeleteReq, BulkDeleteResp, BulkPutReq, BulkPutResp, CommitReq, CommitResp,
    ConfChangeReq, DeleteReq, DeleteResp, GetReq, GetResp, MergeReq, MergeResp, MetricsReq,
    MetricsResp, PeersReq, PeersResp, ProbeReq, ProbeResp, PutReq, PutResp, RaftDone, ReqType,
    RespErr, RollbackReq, RollbackResp, SchemaReq, SchemaResp, SearchReq, SearchResp,
};

pub fn create_client(addr: &str) -> IndexClient {
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(&addr);
    debug!("create channel for {}", addr);
    let index_client = IndexClient::new(ch);
    debug!("create index client for {}", addr);
    index_client
}

pub struct Clerk {
    servers: Vec<IndexClient>,
    client_id: u64,
    request_seq: u64,
    leader_id: usize,
    max_retry_count: usize,
}

impl Clerk {
    pub fn new(servers: &Vec<IndexClient>, client_id: u64) -> Clerk {
        Clerk {
            servers: servers.clone(),
            client_id,
            request_seq: 0,
            leader_id: 0,
            max_retry_count: 5,
        }
    }

    pub fn join(&mut self, id: u64, ip: &str, port: u16) {
        self.join_with_retry(id, ip, port, 5, Duration::from_millis(100))
    }

    pub fn join_with_retry(
        &mut self,
        id: u64,
        ip: &str,
        port: u16,
        max_retry: usize,
        duration: Duration,
    ) {
        let mut cc = ConfChange::new();
        cc.set_id(id);
        cc.set_node_id(id);
        cc.set_change_type(ConfChangeType::AddNode);

        let mut cc_req = ConfChangeReq::new();
        cc_req.set_cc(cc);
        cc_req.set_ip(ip.to_string());
        cc_req.set_port(port as u32);

        let mut request_count: usize = 0;
        loop {
            if request_count > max_retry {
                let msg = "exceeded max retry count";
                debug!("{}", msg);

                let mut ret = HashMap::new();
                ret.insert("error", msg.to_string());

                return;
            }

            let reply = self.servers[self.leader_id]
                .raft_conf_change(&cc_req)
                .unwrap_or_else(|e| {
                    let msg = format!("{:?}", e);
                    error!("{:?}", msg);

                    let mut resp = RaftDone::new();
                    resp.set_err(RespErr::ErrWrongLeader);
                    resp
                });
            match reply.err {
                RespErr::OK => return,
                _ => error!("failed to add node"),
            }

            self.leader_id = (self.leader_id + 1) % self.servers.len();
            request_count += 1;
            debug!("{}", "retry to add node");

            thread::sleep(duration);
        }
    }

    pub fn leave(&mut self, id: u64) {
        let mut cc = ConfChange::new();
        cc.set_id(id);
        cc.set_node_id(id);
        cc.set_change_type(ConfChangeType::RemoveNode);
        let mut cc_req = ConfChangeReq::new();
        cc_req.set_cc(cc);

        let mut request_count: usize = 0;
        loop {
            if request_count > self.max_retry_count {
                let msg = "exceeded max retry count";
                debug!("{}", msg);

                return;
            }

            let reply = self.servers[self.leader_id]
                .raft_conf_change(&cc_req)
                .unwrap_or_else(|e| {
                    let msg = format!("{:?}", e);
                    error!("{:?}", msg);

                    let mut resp = RaftDone::new();
                    resp.set_err(RespErr::ErrWrongLeader);
                    resp
                });
            match reply.err {
                RespErr::OK => return,
                _ => error!("failed to delete from the cluster"),
            }

            self.leader_id = (self.leader_id + 1) % self.servers.len();
            request_count += 1;
            debug!("{}", "retry to add node");

            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn probe(&mut self) -> String {
        let mut req = ProbeReq::new();
        req.set_client_id(self.client_id);
        req.set_seq(self.request_seq);
        self.request_seq += 1;

        let mut request_count: usize = 0;
        loop {
            if request_count > self.max_retry_count {
                let msg = "exceeded max retry count";
                debug!("{}", msg);

                let mut ret = HashMap::new();
                ret.insert("error", msg.to_string());

                return serde_json::to_string(&ret).unwrap();
            }

            let reply = self.servers[self.leader_id]
                .probe(&req)
                .unwrap_or_else(|e| {
                    let msg = format!("{:?}", e);
                    error!("{:?}", msg);

                    let mut ret = HashMap::new();
                    ret.insert("error", msg.to_string());
                    ret.insert("health", "NG".to_string());

                    let mut resp = ProbeResp::new();
                    resp.set_err(RespErr::ErrProbeFailed);
                    resp.set_value(serde_json::to_string(&ret).unwrap());
                    resp
                });
            match reply.err {
                RespErr::OK => return reply.value,
                _ => error!("failed to probe node"),
            }

            self.leader_id = (self.leader_id + 1) % self.servers.len();
            request_count += 1;
            debug!("{}", "retry to probe node");

            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn peers(&mut self) -> String {
        let mut req = PeersReq::new();
        req.set_client_id(self.client_id);
        req.set_seq(self.request_seq);
        self.request_seq += 1;

        let mut request_count: usize = 0;
        loop {
            if request_count > self.max_retry_count {
                let msg = "exceeded max retry count";
                debug!("{}", msg);

                let mut ret = HashMap::new();
                ret.insert("error", msg.to_string());

                return serde_json::to_string(&ret).unwrap();
            }

            let reply = self.servers[self.leader_id]
                .peers(&req)
                .unwrap_or_else(|e| {
                    let msg = format!("{:?}", e);
                    error!("{:?}", msg);

                    let mut ret = HashMap::new();
                    ret.insert("error", msg.to_string());

                    let mut resp = PeersResp::new();
                    resp.set_err(RespErr::ErrPeerFailed);
                    resp.set_value(serde_json::to_string(&ret).unwrap());
                    resp
                });
            match reply.err {
                RespErr::OK => return reply.value,
                _ => error!("failed to get peers"),
            }

            self.leader_id = (self.leader_id + 1) % self.servers.len();
            request_count += 1;
            debug!("{}", "retry to get peers");

            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn metrics(&mut self) -> String {
        let mut req = MetricsReq::new();
        req.set_client_id(self.client_id);
        req.set_seq(self.request_seq);
        self.request_seq += 1;

        let mut request_count: usize = 0;
        loop {
            if request_count > self.max_retry_count {
                let msg = "exceeded max retry count";
                debug!("{}", msg);

                return "{}".to_string();
            }

            let reply = self.servers[self.leader_id]
                .metrics(&req)
                .unwrap_or_else(|e| {
                    let msg = format!("{:?}", e);
                    error!("{:?}", msg);

                    let mut resp = MetricsResp::new();
                    resp.set_err(RespErr::ErrMetricsFailed);
                    resp.set_value("".to_string());
                    resp
                });
            match reply.err {
                RespErr::OK => return reply.value,
                _ => error!("failed to get metrics"),
            }

            self.leader_id = (self.leader_id + 1) % self.servers.len();
            request_count += 1;
            debug!("{}", "retry to get metrics");

            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn get(&mut self, doc_id: &str) -> String {
        let mut req = GetReq::new();
        req.set_client_id(self.client_id);
        req.set_seq(self.request_seq);
        req.set_doc_id(doc_id.to_owned());
        self.request_seq += 1;

        let mut request_count: usize = 0;
        loop {
            if request_count > self.max_retry_count {
                let msg = "exceeded max retry count";
                debug!("{}", msg);

                let mut ret = HashMap::new();
                ret.insert("error", msg.to_string());

                return serde_json::to_string(&ret).unwrap();
            }

            let reply = self.servers[self.leader_id].get(&req).unwrap_or_else(|e| {
                let msg = format!("{:?}", e);
                error!("{:?}", msg);

                let mut ret = HashMap::new();
                ret.insert("error", msg.to_string());

                let mut resp = GetResp::new();
                resp.set_err(RespErr::ErrGetFailed);
                resp.set_value(serde_json::to_string(&ret).unwrap());
                resp
            });
            match reply.err {
                RespErr::OK => return reply.value,
                _ => error!("failed to get document"),
            }

            self.leader_id = (self.leader_id + 1) % self.servers.len();
            request_count += 1;
            debug!("{}", "retry to get document");

            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn put(&mut self, doc: &str) -> String {
        let mut put_req = PutReq::new();
        put_req.set_client_id(self.client_id);
        put_req.set_seq(self.request_seq);
        put_req.set_doc(doc.to_owned());

        let mut req = ApplyReq::new();
        req.set_client_id(self.client_id);
        req.set_req_type(ReqType::Put);
        req.set_put_req(put_req);

        self.request_seq += 1;

        let mut request_count: usize = 0;
        loop {
            if request_count > self.max_retry_count {
                let msg = "exceeded max retry count";
                debug!("{}", msg);

                let mut ret = HashMap::new();
                ret.insert("error", msg.to_string());

                return serde_json::to_string(&ret).unwrap();
            }

            let reply = self.servers[self.leader_id].put(&req).unwrap_or_else(|e| {
                let msg = format!("{:?}", e);
                error!("{:?}", msg);

                let mut ret = HashMap::new();
                ret.insert("error", msg.to_string());

                let mut resp = PutResp::new();
                resp.set_err(RespErr::ErrPutFailed);
                resp.set_value(serde_json::to_string(&ret).unwrap());
                resp
            });
            match reply.err {
                RespErr::OK => return reply.value,
                _ => error!("failed to put document"),
            }

            self.leader_id = (self.leader_id + 1) % self.servers.len();
            request_count += 1;
            debug!("{}", "retry to put document");

            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn delete(&mut self, doc_id: &str) -> String {
        let mut delete_req = DeleteReq::new();
        delete_req.set_client_id(self.client_id);
        delete_req.set_seq(self.request_seq);
        delete_req.set_doc_id(doc_id.to_owned());

        let mut req = ApplyReq::new();
        req.set_client_id(self.client_id);
        req.set_req_type(ReqType::Delete);
        req.set_delete_req(delete_req);

        self.request_seq += 1;

        let mut request_count: usize = 0;
        loop {
            if request_count > self.max_retry_count {
                let msg = "exceeded max retry count";
                debug!("{}", msg);

                let mut ret = HashMap::new();
                ret.insert("error", msg.to_string());

                return serde_json::to_string(&ret).unwrap();
            }

            let reply = self.servers[self.leader_id]
                .delete(&req)
                .unwrap_or_else(|e| {
                    let msg = format!("{:?}", e);
                    error!("{:?}", msg);

                    let mut ret = HashMap::new();
                    ret.insert("error", msg.to_string());

                    let mut resp = DeleteResp::new();
                    resp.set_err(RespErr::ErrDeleteFailed);
                    resp.set_value(serde_json::to_string(&ret).unwrap());
                    resp
                });
            match reply.err {
                RespErr::OK => return reply.value,
                _ => error!("failed to delete document"),
            }

            self.leader_id = (self.leader_id + 1) % self.servers.len();
            request_count += 1;
            debug!("{}", "retry to delete document");

            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn bulk_put(&mut self, docs: &str) -> String {
        let mut bulk_put_req = BulkPutReq::new();
        bulk_put_req.set_client_id(self.client_id);
        bulk_put_req.set_seq(self.request_seq);
        bulk_put_req.set_docs(docs.to_owned());

        let mut req = ApplyReq::new();
        req.set_client_id(self.client_id);
        req.set_req_type(ReqType::BulkPut);
        req.set_bulk_put_req(bulk_put_req);

        self.request_seq += 1;

        let mut request_count: usize = 0;
        loop {
            if request_count > self.max_retry_count {
                let msg = "exceeded max retry count";
                debug!("{}", msg);

                let mut ret = HashMap::new();
                ret.insert("error", msg.to_string());

                return serde_json::to_string(&ret).unwrap();
            }

            let reply = self.servers[self.leader_id]
                .bulk_put(&req)
                .unwrap_or_else(|e| {
                    let msg = format!("{:?}", e);
                    error!("{:?}", msg);

                    let mut ret = HashMap::new();
                    ret.insert("error", msg.to_string());

                    let mut resp = BulkPutResp::new();
                    resp.set_err(RespErr::ErrBulkPutFailed);
                    resp.set_value(serde_json::to_string(&ret).unwrap());
                    resp
                });
            match reply.err {
                RespErr::OK => return reply.value,
                _ => error!("failed to put documents in bulk"),
            }

            self.leader_id = (self.leader_id + 1) % self.servers.len();
            request_count += 1;
            debug!("{}", "retry to put documents in bulk");

            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn bulk_delete(&mut self, docs: &str) -> String {
        let mut bulk_delete_req = BulkDeleteReq::new();
        bulk_delete_req.set_client_id(self.client_id);
        bulk_delete_req.set_seq(self.request_seq);
        bulk_delete_req.set_docs(docs.to_owned());

        let mut req = ApplyReq::new();
        req.set_client_id(self.client_id);
        req.set_req_type(ReqType::BulkDelete);
        req.set_bulk_delete_req(bulk_delete_req);

        self.request_seq += 1;

        let mut request_count: usize = 0;
        loop {
            if request_count > self.max_retry_count {
                let msg = "exceeded max retry count";
                debug!("{}", msg);

                let mut ret = HashMap::new();
                ret.insert("error", msg.to_string());

                return serde_json::to_string(&ret).unwrap();
            }

            let reply = self.servers[self.leader_id]
                .bulk_delete(&req)
                .unwrap_or_else(|e| {
                    let msg = format!("{:?}", e);
                    error!("{:?}", msg);

                    let mut ret = HashMap::new();
                    ret.insert("error", msg.to_string());

                    let mut resp = BulkDeleteResp::new();
                    resp.set_err(RespErr::ErrBulkDeleteFailed);
                    resp.set_value(serde_json::to_string(&ret).unwrap());
                    resp
                });
            match reply.err {
                RespErr::OK => return reply.value,
                _ => error!("failed to delete documents in bulk"),
            }

            self.leader_id = (self.leader_id + 1) % self.servers.len();
            request_count += 1;
            debug!("{}", "retry to delete documents in bulk");

            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn commit(&mut self) -> String {
        let mut commit_req = CommitReq::new();
        commit_req.set_client_id(self.client_id);
        commit_req.set_seq(self.request_seq);

        let mut req = ApplyReq::new();
        req.set_client_id(self.client_id);
        req.set_req_type(ReqType::Commit);
        req.set_commit_req(commit_req);

        self.request_seq += 1;

        let mut request_count: usize = 0;
        loop {
            if request_count > self.max_retry_count {
                let msg = "exceeded max retry count";
                debug!("{}", msg);

                let mut ret = HashMap::new();
                ret.insert("error", msg.to_string());

                return serde_json::to_string(&ret).unwrap();
            }

            let reply = self.servers[self.leader_id]
                .commit(&req)
                .unwrap_or_else(|e| {
                    let msg = format!("{:?}", e);
                    error!("{:?}", msg);

                    let mut ret = HashMap::new();
                    ret.insert("error", msg.to_string());

                    let mut resp = CommitResp::new();
                    resp.set_err(RespErr::ErrCommitFailed);
                    resp.set_value(serde_json::to_string(&ret).unwrap());
                    resp
                });
            match reply.err {
                RespErr::OK => {
                    debug!("commit succeeded");
                    return reply.value;
                }
                RespErr::ErrWrongLeader => error!("wrong leader"),
                RespErr::ErrTimeout => error!("timeout"),
                RespErr::ErrDisconnected => error!("disconnected"),
                _ => error!("failed to commit index"),
            }
            self.leader_id = (self.leader_id + 1) % self.servers.len();
            request_count += 1;
            debug!("{}", "retry to commit index");

            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn rollback(&mut self) -> String {
        let mut rollback_req = RollbackReq::new();
        rollback_req.set_client_id(self.client_id);
        rollback_req.set_seq(self.request_seq);

        let mut req = ApplyReq::new();
        req.set_client_id(self.client_id);
        req.set_req_type(ReqType::Rollback);
        req.set_rollback_req(rollback_req);

        self.request_seq += 1;

        let mut request_count: usize = 0;
        loop {
            if request_count > self.max_retry_count {
                let msg = "exceeded max retry count";
                debug!("{}", msg);

                let mut ret = HashMap::new();
                ret.insert("error", msg.to_string());

                return serde_json::to_string(&ret).unwrap();
            }

            let reply = self.servers[self.leader_id]
                .rollback(&req)
                .unwrap_or_else(|e| {
                    let msg = format!("{:?}", e);
                    error!("{:?}", msg);

                    let mut ret = HashMap::new();
                    ret.insert("error", msg.to_string());

                    let mut resp = RollbackResp::new();
                    resp.set_err(RespErr::ErrRollbackFailed);
                    resp.set_value(serde_json::to_string(&ret).unwrap());
                    resp
                });
            match reply.err {
                RespErr::OK => return reply.value,
                _ => error!("failed to rollback index"),
            }

            self.leader_id = (self.leader_id + 1) % self.servers.len();
            request_count += 1;
            debug!("{}", "retry to rollback index");

            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn merge(&mut self) -> String {
        let mut merge_req = MergeReq::new();
        merge_req.set_client_id(self.client_id);
        merge_req.set_seq(self.request_seq);

        let mut req = ApplyReq::new();
        req.set_client_id(self.client_id);
        req.set_req_type(ReqType::Merge);
        req.set_merge_req(merge_req);

        self.request_seq += 1;

        let mut request_count: usize = 0;
        loop {
            if request_count > self.max_retry_count {
                let msg = "exceeded max retry count";
                debug!("{}", msg);

                let mut ret = HashMap::new();
                ret.insert("error", msg.to_string());

                return serde_json::to_string(&ret).unwrap();
            }

            let reply = self.servers[self.leader_id]
                .merge(&req)
                .unwrap_or_else(|e| {
                    let msg = format!("{:?}", e);
                    error!("{:?}", msg);

                    let mut ret = HashMap::new();
                    ret.insert("error", msg.to_string());

                    let mut resp = MergeResp::new();
                    resp.set_err(RespErr::ErrMergeFailed);
                    resp.set_value(serde_json::to_string(&ret).unwrap());
                    resp
                });
            match reply.err {
                RespErr::OK => return reply.value,
                _ => error!("failed to merge index"),
            }

            self.leader_id = (self.leader_id + 1) % self.servers.len();
            request_count += 1;
            debug!("{}", "retry to merge index");

            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn search(
        &mut self,
        query: &str,
        from: u64,
        limit: u64,
        exclude_count: bool,
        exclude_docs: bool,
        facet_field: &str,
        facet_prefixes: Vec<String>,
    ) -> String {
        let mut req = SearchReq::new();
        req.set_client_id(self.client_id);
        req.set_seq(self.request_seq);
        req.set_query(query.to_owned());
        req.set_from(from);
        req.set_limit(limit);
        req.set_exclude_count(exclude_count);
        req.set_exclude_docs(exclude_docs);
        req.set_facet_field(facet_field.to_string());
        req.set_facet_prefixes(RepeatedField::from_vec(facet_prefixes));
        self.request_seq += 1;

        let mut request_count: usize = 0;
        loop {
            if request_count > self.max_retry_count {
                let msg = "exceeded max retry count";
                debug!("{}", msg);

                let mut ret = HashMap::new();
                ret.insert("error", msg.to_string());

                return serde_json::to_string(&ret).unwrap();
            }

            let reply = self.servers[self.leader_id]
                .search(&req)
                .unwrap_or_else(|e| {
                    let msg = format!("{:?}", e);
                    error!("{:?}", msg);

                    let mut ret = HashMap::new();
                    ret.insert("error", msg.to_string());

                    let mut resp = SearchResp::new();
                    resp.set_err(RespErr::ErrSearchFailed);
                    resp.set_value(serde_json::to_string(&ret).unwrap());
                    resp
                });
            match reply.err {
                RespErr::OK => return reply.value,
                _ => error!("failed to search index"),
            }

            self.leader_id = (self.leader_id + 1) % self.servers.len();
            request_count += 1;
            debug!("{}", "retry to search index");

            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn schema(&mut self) -> String {
        let mut req = SchemaReq::new();
        req.set_client_id(self.client_id);
        req.set_seq(self.request_seq);
        self.request_seq += 1;

        let mut request_count: usize = 0;
        loop {
            if request_count > self.max_retry_count {
                let msg = "exceeded max retry count";
                debug!("{}", msg);

                let mut ret = HashMap::new();
                ret.insert("error", msg.to_string());

                return serde_json::to_string(&ret).unwrap();
            }

            let reply = self.servers[self.leader_id]
                .schema(&req)
                .unwrap_or_else(|e| {
                    let msg = format!("{:?}", e);
                    error!("{:?}", msg);

                    let mut ret = HashMap::new();
                    ret.insert("error", msg.to_string());

                    let mut resp = SchemaResp::new();
                    resp.set_err(RespErr::ErrSchemaFailed);
                    resp.set_value(serde_json::to_string(&ret).unwrap());
                    resp
                });
            match reply.err {
                RespErr::OK => return reply.value,
                _ => error!("failed to get index schema"),
            }

            self.leader_id = (self.leader_id + 1) % self.servers.len();
            request_count += 1;
            debug!("{}", "retry to get index schema");

            thread::sleep(Duration::from_millis(100));
        }
    }
}
