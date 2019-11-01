use crate::proto::indexpb_grpc::IndexClient;
use crate::proto::indexrpcpb::{ConfChangeReq, RaftDone, RespErr};
use log::*;
use protobuf::{self, Message};
use raft::Config;
use std::collections::HashMap;

pub fn default_raft_config(id: u64, peers: Vec<u64>) -> Config {
    debug!("default_raft_config id:{} peers:{:?}", id, peers);
    Config {
        id,
        peers,
        election_tick: 10,
        heartbeat_tick: 1,
        max_size_per_msg: 1024 * 1024 * 1024,
        max_inflight_msgs: 256,
        applied: 0,
        ..Default::default()
    }
}

pub fn parse_data<T: Message>(data: &[u8]) -> T {
    protobuf::parse_from_bytes::<T>(data).unwrap_or_else(|e| {
        panic!("data is corrupted: {:?}", e);
    })
}

pub fn conf_change(
    self_id: u64,
    leader_id: u64,
    peers: &HashMap<u64, IndexClient>,
    req: ConfChangeReq,
) {
    let client = peers.get(&leader_id).unwrap();
    let reply = client.raft_conf_change(&req).unwrap_or_else(|_e| {
        let mut resp = RaftDone::new();
        resp.set_err(RespErr::ErrWrongLeader);
        resp
    });
    match reply.err {
        RespErr::OK => return,
        RespErr::ErrWrongLeader => (),
        RespErr::ErrNoKey => return,
    }
    loop {
        for (id, client) in peers.iter() {
            if *id != self_id {
                let reply = client.raft_conf_change(&req).unwrap_or_else(|_e| {
                    let mut resp = RaftDone::new();
                    resp.set_err(RespErr::ErrWrongLeader);
                    resp
                });
                match reply.err {
                    RespErr::OK => return,
                    RespErr::ErrWrongLeader => (),
                    RespErr::ErrNoKey => return,
                }
            }
        }
    }
}
