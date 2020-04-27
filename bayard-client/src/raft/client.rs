use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::sync::Arc;

use bincode::{deserialize, serialize};
use grpcio::{ChannelBuilder, EnvBuilder};
use log::*;
use raft::eraftpb::{ConfChange, ConfChangeType};

use bayard_proto::proto::commonpb::State;
use bayard_proto::proto::raftpb_grpc::RaftServiceClient;
use bayard_server::raft::config::NodeAddress;

fn create_client(address: String) -> RaftServiceClient {
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(&address);
    let client = RaftServiceClient::new(ch);
    client
}

pub struct RaftClient {
    leader_id: u64, // leader's index in server_ids
    clients: HashMap<u64, Arc<RaftServiceClient>>,
    addresses: HashMap<u64, String>,
}

impl RaftClient {
    pub fn new(address: &str) -> RaftClient {
        let initial_leader_id = 0;

        let mut addresses = HashMap::new();
        addresses.insert(initial_leader_id, address.to_string());

        let mut clients = HashMap::new();
        let client = create_client(address.to_string());
        clients.insert(initial_leader_id, Arc::new(client));

        RaftClient {
            leader_id: initial_leader_id,
            clients,
            addresses,
        }
    }

    pub fn join(
        &mut self,
        id: u64,
        node_address: NodeAddress,
    ) -> Result<HashMap<u64, NodeAddress>, std::io::Error> {
        let mut req = ConfChange::new();
        req.set_node_id(id);
        req.set_change_type(ConfChangeType::AddNode);
        req.set_context(serialize(&node_address).unwrap());

        let max_retry = 10;
        let mut cnt_retry = 0;

        loop {
            if max_retry < cnt_retry {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("max retry count has been exceeded: max_retry={}", max_retry),
                ));
            }

            let client = match self.clients.get(&self.leader_id) {
                Some(c) => c,
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("failed to get client for node: id={}", self.leader_id),
                    ));
                }
            };

            let reply = match client.change_config(&req) {
                Ok(r) => r,
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!(
                            "failed to join node to the cluster: id={}",
                            req.get_node_id()
                        ),
                    ));
                }
            };

            // update address list and clients
            if reply.get_address_map().len() > 0 {
                let address_map: HashMap<u64, NodeAddress> =
                    deserialize(&reply.get_address_map()).unwrap();
                // add new ids
                for (id, address) in &address_map {
                    if let Some(grpc_address) = self.addresses.get(&id) {
                        if grpc_address == address.raft_address.as_str() {
                            debug!(
                                "node has not been changed: id={}, address={}",
                                id, grpc_address
                            );
                        } else {
                            debug!("update node: id={}, address={}", id, address.raft_address);
                            self.addresses
                                .insert(id.clone(), address.raft_address.clone());
                            self.clients.insert(
                                id.clone(),
                                Arc::new(create_client(address.raft_address.clone())),
                            );
                        }
                    } else {
                        debug!("add node: id={}, address={}", id, address.raft_address);
                        self.addresses
                            .insert(id.clone(), address.raft_address.clone());
                        self.clients.insert(
                            id.clone(),
                            Arc::new(create_client(address.raft_address.clone())),
                        );
                    }
                }

                // remove unused ids
                for (id, address) in &self.addresses.clone() {
                    if let Some(_) = address_map.get(&id) {
                        debug!("node is in use: id={}, address={}", id, address);
                    } else {
                        debug!("node is not in use: id={}, address={}", id, address);
                        self.addresses.remove(id);
                        self.clients.remove(id);
                    }
                }

                debug!("addresses={:?}", self.addresses);
            }

            match reply.get_state() {
                State::OK => {
                    return Ok(deserialize(&reply.get_address_map()).unwrap());
                }
                State::WRONG_LEADER => {
                    warn!(
                        "upddate leader id: current={}, new={}",
                        self.leader_id,
                        reply.get_leader_id()
                    );
                    self.leader_id = reply.get_leader_id();
                    cnt_retry += 1;
                    warn!("retry with a new leader: id={}", self.leader_id);
                    continue;
                }
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!(
                            "failed to join node to the cluster: id={}",
                            req.get_node_id()
                        ),
                    ));
                }
            };
        }
    }

    pub fn leave(&mut self, id: u64) -> Result<HashMap<u64, NodeAddress>, std::io::Error> {
        let mut req = ConfChange::new();
        req.set_node_id(id);
        req.set_change_type(ConfChangeType::RemoveNode);
        req.set_context(vec![]);

        let max_retry = 10;
        let mut cnt_retry = 0;

        loop {
            if max_retry < cnt_retry {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("max retry count has been exceeded: max_retry={}", max_retry),
                ));
            }

            let client = match self.clients.get(&self.leader_id) {
                Some(c) => c,
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("failed to get client for node: id={}", self.leader_id),
                    ));
                }
            };

            let reply = match client.change_config(&req) {
                Ok(r) => r,
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!(
                            "failed to leave node from the cluster: id={}",
                            req.get_node_id()
                        ),
                    ));
                }
            };

            // update address list and clients
            if reply.get_address_map().len() > 0 {
                let address_map: HashMap<u64, NodeAddress> =
                    deserialize(&reply.get_address_map()).unwrap();
                // add new ids
                for (id, address) in &address_map {
                    if let Some(grpc_address) = self.addresses.get(&id) {
                        if grpc_address == address.raft_address.as_str() {
                            debug!(
                                "node has not been changed: id={}, address={}",
                                id, grpc_address
                            );
                        } else {
                            debug!("update node: id={}, address={}", id, address.raft_address);
                            self.addresses
                                .insert(id.clone(), address.raft_address.clone());
                            self.clients.insert(
                                id.clone(),
                                Arc::new(create_client(address.raft_address.clone())),
                            );
                        }
                    } else {
                        debug!("add node: id={}, address={}", id, address.raft_address);
                        self.addresses
                            .insert(id.clone(), address.raft_address.clone());
                        self.clients.insert(
                            id.clone(),
                            Arc::new(create_client(address.raft_address.clone())),
                        );
                    }
                }

                // remove unused ids
                for (id, address) in &self.addresses.clone() {
                    if let Some(_) = address_map.get(&id) {
                        debug!("node is in use: id={}, address={}", id, address);
                    } else {
                        debug!("node is not in use: id={}, address={}", id, address);
                        self.addresses.remove(id);
                        self.clients.remove(id);
                    }
                }

                debug!("addresses={:?}", self.addresses);
            }

            match reply.get_state() {
                State::OK => {
                    return Ok(deserialize(&reply.get_address_map()).unwrap());
                }
                State::WRONG_LEADER => {
                    warn!(
                        "upddate leader id: current={}, new={}",
                        self.leader_id,
                        reply.get_leader_id()
                    );
                    self.leader_id = reply.get_leader_id();
                    cnt_retry += 1;
                    warn!("retry with a new leader: id={}", self.leader_id);
                    continue;
                }
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!(
                            "failed to leave node from the cluster: id={}",
                            req.get_node_id()
                        ),
                    ));
                }
            };
        }
    }

    pub fn snapshot(&mut self) -> Result<(), std::io::Error> {
        // TODO
        Ok(())
    }
}
