use std::sync::Arc;
use std::thread;
use std::time::Duration;

use grpcio::{ChannelBuilder, EnvBuilder};
use log::*;

use crate::proto::indexpb_grpc::IndexClient;
use crate::proto::indexrpcpb::{
    DeleteResp, GetResp, IndexReq, PutResp, ReqType, RespErr, SearchResp,
};

pub fn create_client(addr: &str) -> IndexClient {
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(&addr);
    debug!("create index client for {}", addr);
    IndexClient::new(ch)
}

pub struct Clerk {
    servers: Vec<IndexClient>,
    client_id: u64,
    request_seq: u64,
    leader_id: usize,
}

impl Clerk {
    pub fn new(servers: &Vec<IndexClient>, client_id: u64) -> Clerk {
        Clerk {
            servers: servers.clone(),
            client_id,
            request_seq: 0,
            leader_id: 0,
        }
    }

    pub fn get(&mut self, key: &str) -> String {
        let mut req = IndexReq::new();
        req.set_client_id(self.client_id);
        req.set_req_type(ReqType::Get);
        req.set_seq(self.request_seq);
        req.set_key(key.to_owned());
        self.request_seq += 1;

        loop {
            let reply = self.servers[self.leader_id].get(&req).unwrap_or_else(|_e| {
                let mut resp = GetResp::new();
                resp.set_err(RespErr::ErrWrongLeader);
                resp
            });
            match reply.err {
                RespErr::OK => return reply.value,
                RespErr::ErrWrongLeader => (),
                RespErr::ErrNoKey => return String::from(""),
            }
            self.leader_id = (self.leader_id + 1) % self.servers.len();
            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn put(&mut self, key: &str, value: &str) {
        let mut req = IndexReq::new();
        req.set_client_id(self.client_id);
        req.set_req_type(ReqType::Put);
        req.set_seq(self.request_seq);
        req.set_key(key.to_owned());
        req.set_value(value.to_owned());
        self.request_seq += 1;

        loop {
            let reply = self.servers[self.leader_id].put(&req).unwrap_or_else(|_e| {
                let mut resp = PutResp::new();
                resp.set_err(RespErr::ErrWrongLeader);
                resp
            });
            match reply.err {
                RespErr::OK => return,
                RespErr::ErrWrongLeader => (),
                RespErr::ErrNoKey => return,
            }
            debug!("put redo: {}", self.leader_id);
            self.leader_id = (self.leader_id + 1) % self.servers.len();
            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn delete(&mut self, key: &str) {
        let mut req = IndexReq::new();
        req.set_client_id(self.client_id);
        req.set_req_type(ReqType::Delete);
        req.set_seq(self.request_seq);
        req.set_key(key.to_owned());
        self.request_seq += 1;

        loop {
            let reply = self.servers[self.leader_id]
                .delete(&req)
                .unwrap_or_else(|_e| {
                    let mut resp = DeleteResp::new();
                    resp.set_err(RespErr::ErrWrongLeader);
                    resp
                });
            match reply.err {
                RespErr::OK => return,
                RespErr::ErrWrongLeader => (),
                RespErr::ErrNoKey => return,
            }
            self.leader_id = (self.leader_id + 1) % self.servers.len();
            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn search(&mut self, query: &str) -> String {
        let mut req = IndexReq::new();
        req.set_client_id(self.client_id);
        req.set_req_type(ReqType::Search);
        req.set_seq(self.request_seq);
        req.set_query(query.to_owned());
        self.request_seq += 1;

        loop {
            let reply = self.servers[self.leader_id]
                .search(&req)
                .unwrap_or_else(|_e| {
                    let mut resp = SearchResp::new();
                    resp.set_err(RespErr::ErrWrongLeader);
                    resp
                });
            match reply.err {
                RespErr::OK => return reply.value,
                RespErr::ErrWrongLeader => (),
                RespErr::ErrNoKey => return String::from(""),
            }
            self.leader_id = (self.leader_id + 1) % self.servers.len();
            thread::sleep(Duration::from_millis(100));
        }
    }
}
