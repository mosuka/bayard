use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::time::Duration;

use futures::Future;
use grpcio::{RpcContext, UnarySink};
use log::*;
use raft::eraftpb::{ConfChange, Message, Snapshot};

use bayard_proto::proto::commonpb::State;
use bayard_proto::proto::raftpb::{AddressState, ChangeReply, Null};
use bayard_proto::proto::raftpb_grpc::RaftService;

use crate::raft::config;

#[derive(Clone)]
pub struct RaftServer {
    pub sender: Sender<config::Msg>,
    seq: u64,
}

impl RaftServer {
    pub fn new(sender: Sender<config::Msg>) -> RaftServer {
        RaftServer { sender, seq: 0 }
    }
}

impl RaftService for RaftServer {
    fn snapshot(&mut self, _ctx: RpcContext, _req: Snapshot, _sink: UnarySink<Null>) {
        // TODO:
    }

    fn change_config(&mut self, ctx: RpcContext, req: ConfChange, sink: UnarySink<ChangeReply>) {
        debug!("change config");
        let (s1, r1) = mpsc::channel();
        let sender = self.sender.clone();
        let seq = self.seq;
        self.seq += 1;
        sender
            .send(config::Msg::ConfigChange {
                seq,
                change: req,
                cb: Box::new(move |leader_id: i32, addresses: Vec<u8>| {
                    let mut reply = ChangeReply::new();
                    if leader_id >= 0 {
                        reply.set_state(State::WRONG_LEADER);
                        reply.set_leader_id(leader_id as u64);
                    } else {
                        reply.set_state(State::OK);
                    }
                    reply.set_address_map(addresses);
                    s1.send(reply).expect("cb channel closed");
                }),
            })
            .unwrap();
        let reply = match r1.recv_timeout(Duration::from_secs(2)) {
            Ok(r) => r,
            Err(e) => {
                warn!("timeout: {:?}", e);
                let mut r = ChangeReply::new();
                r.set_state(State::IO_ERROR);
                r
            }
        };
        let f = sink
            .success(reply.clone())
            .map_err(move |err| error!("Failed to reply put: {:?}", err));
        ctx.spawn(f);
    }

    fn send_msg(&mut self, _ctx: RpcContext, req: Message, _sink: ::grpcio::UnarySink<Null>) {
        let sender = self.sender.clone();
        sender.send(config::Msg::Raft(req)).unwrap();
    }

    fn send_address(&mut self, _ctx: RpcContext, req: AddressState, _sink: UnarySink<Null>) {
        let sender = self.sender.clone();
        sender.send(config::Msg::Address(req)).unwrap();
    }
}
