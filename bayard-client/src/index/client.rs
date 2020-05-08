use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::sync::Arc;

use bayard_proto::proto::commonpb::State;
use bayard_proto::proto::indexpb::{
    BulkDeleteReq, BulkSetReq, CommitReq, DeleteReq, GetReq, MergeReq, RollbackReq, SchemaReq,
    SearchReq, SetReq, StatusReq,
};
use bayard_proto::proto::indexpb_grpc::IndexServiceClient;
use bincode::deserialize;
use grpcio::{ChannelBuilder, EnvBuilder};
use log::*;
use protobuf::RepeatedField;
use serde_json::json;
use serde_json::Value;

use bayard_server::raft::config::NodeAddress;

fn create_client(address: String) -> IndexServiceClient {
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(&address);
    let client = IndexServiceClient::new(ch);
    client
}

#[derive(Clone)]
pub struct IndexClient {
    address: String,
    leader_id: u64,
    clients: HashMap<u64, Arc<IndexServiceClient>>,
    addresses: HashMap<u64, String>,
    next_index: usize,
    node_id: u64,
    client_id: u64,
}

impl IndexClient {
    pub fn new(address: &str) -> IndexClient {
        let initial_id = 0; // initial node id

        let mut addresses = HashMap::new();
        addresses.insert(initial_id, address.to_string());

        let mut clients = HashMap::new();
        let client = create_client(address.to_string());
        clients.insert(initial_id, Arc::new(client));

        IndexClient {
            address: address.to_string(),
            leader_id: initial_id,
            clients,
            addresses,
            next_index: 0,
            node_id: initial_id,
            client_id: rand::random(),
        }
    }

    pub fn get(&mut self, id: String) -> Result<String, std::io::Error> {
        let mut req = GetReq::new();
        req.set_client_id(self.client_id);
        req.set_id(id);

        let max_retry = 10;
        let mut cnt_retry = 0;

        loop {
            if max_retry < cnt_retry {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("max retry count has been exceeded: max_retry={}", max_retry),
                ));
            }

            let client = match self.clients.get(&self.node_id) {
                Some(c) => c,
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("failed to get client for node: id={}", self.node_id),
                    ));
                }
            };

            let reply = match client.get(&req) {
                Ok(r) => r,
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("failed to request getting document: id={}", req.get_id()),
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
                        if grpc_address == address.index_address.as_str() {
                            debug!(
                                "node has not been changed: id={}, address={}",
                                id, grpc_address
                            );
                        } else {
                            debug!("update node: id={}, address={}", id, address.index_address);
                            self.addresses
                                .insert(id.clone(), address.index_address.clone());
                            self.clients.insert(
                                id.clone(),
                                Arc::new(create_client(address.index_address.clone())),
                            );
                        }
                    } else {
                        debug!("add node: id={}, address={}", id, address.index_address);
                        self.addresses
                            .insert(id.clone(), address.index_address.clone());
                        self.clients.insert(
                            id.clone(),
                            Arc::new(create_client(address.index_address.clone())),
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

            // change node id
            let keys: Vec<u64> = self.addresses.keys().map(|i| i.clone()).collect();
            self.next_index = (self.next_index + 1) % self.addresses.len();
            self.node_id = keys.get(self.next_index).unwrap().clone();

            match reply.get_state() {
                State::OK => {
                    self.leader_id = reply.get_leader_id();
                    return Ok(String::from(reply.get_doc()));
                }
                State::NOT_FOUND => {
                    self.leader_id = reply.get_leader_id();
                    return Err(Error::new(
                        ErrorKind::NotFound,
                        format!("document not found: id={}", req.get_id()),
                    ));
                }
                _ => {
                    cnt_retry += 1;
                    warn!("failed to get document: id={}", req.get_id());
                }
            }
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
    ) -> Result<String, std::io::Error> {
        let mut req = SearchReq::new();
        req.set_client_id(self.client_id);
        req.set_query(query.to_string());
        req.set_from(from);
        req.set_limit(limit);
        req.set_exclude_count(exclude_count);
        req.set_exclude_docs(exclude_docs);
        req.set_facet_field(facet_field.to_string());
        req.set_facet_prefixes(RepeatedField::from_vec(facet_prefixes));

        let max_retry = 10;
        let mut cnt_retry = 0;

        loop {
            if max_retry < cnt_retry {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("max retry count has been exceeded: max_retry={}", max_retry),
                ));
            }

            let client = match self.clients.get(&self.node_id) {
                Some(c) => c,
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("failed to get client for node: id={}", self.node_id),
                    ));
                }
            };

            let reply = match client.search(&req) {
                Ok(r) => r,
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("failed to request searching documents: req={:?}", req),
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
                        if grpc_address == address.index_address.as_str() {
                            debug!(
                                "node has not been changed: id={}, address={}",
                                id, grpc_address
                            );
                        } else {
                            debug!("update node: id={}, address={}", id, address.index_address);
                            self.addresses
                                .insert(id.clone(), address.index_address.clone());
                            self.clients.insert(
                                id.clone(),
                                Arc::new(create_client(address.index_address.clone())),
                            );
                        }
                    } else {
                        debug!("add node: id={}, address={}", id, address.index_address);
                        self.addresses
                            .insert(id.clone(), address.index_address.clone());
                        self.clients.insert(
                            id.clone(),
                            Arc::new(create_client(address.index_address.clone())),
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

            // change node id
            let keys: Vec<u64> = self.addresses.keys().map(|i| i.clone()).collect();
            self.next_index = (self.next_index + 1) % self.addresses.len();
            self.node_id = keys.get(self.next_index).unwrap().clone();

            match reply.get_state() {
                State::OK => {
                    return Ok(String::from(reply.get_search_result()));
                }
                _ => {
                    cnt_retry += 1;
                    warn!("failed to search documents: req={:?}", req);
                }
            }
        }
    }

    pub fn set(&mut self, doc: String) -> Result<(), std::io::Error> {
        let mut req = SetReq::new();
        req.set_client_id(self.client_id);
        req.set_doc(doc);

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

            let reply = match client.set(&req) {
                Ok(r) => r,
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("failed to request setting document: doc={}", req.get_doc()),
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
                        if grpc_address == address.index_address.as_str() {
                            debug!(
                                "node has not been changed: id={}, address={}",
                                id, grpc_address
                            );
                        } else {
                            debug!("update node: id={}, address={}", id, address.index_address);
                            self.addresses
                                .insert(id.clone(), address.index_address.clone());
                            self.clients.insert(
                                id.clone(),
                                Arc::new(create_client(address.index_address.clone())),
                            );
                        }
                    } else {
                        debug!("add node: id={}, address={}", id, address.index_address);
                        self.addresses
                            .insert(id.clone(), address.index_address.clone());
                        self.clients.insert(
                            id.clone(),
                            Arc::new(create_client(address.index_address.clone())),
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
                    self.leader_id = reply.get_leader_id();
                    return Ok(());
                }
                State::WRONG_LEADER => {
                    self.leader_id = reply.get_leader_id();
                    cnt_retry += 1;
                    debug!("retry with a new leader: id={}", self.leader_id);
                    continue;
                }
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("failed to set document: doc={}", req.get_doc()),
                    ));
                }
            };
        }
    }

    pub fn delete(&mut self, id: String) -> Result<(), std::io::Error> {
        let mut req = DeleteReq::new();
        req.set_client_id(self.client_id);
        req.set_id(id);

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

            let reply = match client.delete(&req) {
                Ok(r) => r,
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("failed to request deleting document: id={}", req.get_id()),
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
                        if grpc_address == address.index_address.as_str() {
                            debug!(
                                "node has not been changed: id={}, address={}",
                                id, grpc_address
                            );
                        } else {
                            debug!("update node: id={}, address={}", id, address.index_address);
                            self.addresses
                                .insert(id.clone(), address.index_address.clone());
                            self.clients.insert(
                                id.clone(),
                                Arc::new(create_client(address.index_address.clone())),
                            );
                        }
                    } else {
                        debug!("add node: id={}, address={}", id, address.index_address);
                        self.addresses
                            .insert(id.clone(), address.index_address.clone());
                        self.clients.insert(
                            id.clone(),
                            Arc::new(create_client(address.index_address.clone())),
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
                    self.leader_id = reply.get_leader_id();
                    return Ok(());
                }
                State::WRONG_LEADER => {
                    self.leader_id = reply.get_leader_id();
                    cnt_retry += 1;
                    debug!("retry with a new leader: id={}", self.leader_id);
                    continue;
                }
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("failed to delete document: id={}", req.get_id()),
                    ));
                }
            };
        }
    }

    pub fn bulk_set(&mut self, docs: String) -> Result<(), std::io::Error> {
        let mut req = BulkSetReq::new();
        req.set_client_id(self.client_id);
        req.set_docs(docs);

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

            let reply = match client.bulk_set(&req) {
                Ok(r) => r,
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "failed to request setting documents in bulk",
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
                        if grpc_address == address.index_address.as_str() {
                            debug!(
                                "node has not been changed: id={}, address={}",
                                id, grpc_address
                            );
                        } else {
                            debug!("update node: id={}, address={}", id, address.index_address);
                            self.addresses
                                .insert(id.clone(), address.index_address.clone());
                            self.clients.insert(
                                id.clone(),
                                Arc::new(create_client(address.index_address.clone())),
                            );
                        }
                    } else {
                        debug!("add node: id={}, address={}", id, address.index_address);
                        self.addresses
                            .insert(id.clone(), address.index_address.clone());
                        self.clients.insert(
                            id.clone(),
                            Arc::new(create_client(address.index_address.clone())),
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
                    self.leader_id = reply.get_leader_id();
                    return Ok(());
                }
                State::WRONG_LEADER => {
                    self.leader_id = reply.get_leader_id();
                    cnt_retry += 1;
                    debug!("retry with a new leader: id={}", self.leader_id);
                    continue;
                }
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "failed to set documents in bulk",
                    ));
                }
            };
        }
    }

    pub fn bulk_delete(&mut self, docs: String) -> Result<(), std::io::Error> {
        let mut req = BulkDeleteReq::new();
        req.set_client_id(self.client_id);
        req.set_docs(docs);

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

            let reply = match client.bulk_delete(&req) {
                Ok(r) => r,
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "failed to request deleting documents in bulk",
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
                        if grpc_address == address.index_address.as_str() {
                            debug!(
                                "node has not been changed: id={}, address={}",
                                id, grpc_address
                            );
                        } else {
                            debug!("update node: id={}, address={}", id, address.index_address);
                            self.addresses
                                .insert(id.clone(), address.index_address.clone());
                            self.clients.insert(
                                id.clone(),
                                Arc::new(create_client(address.index_address.clone())),
                            );
                        }
                    } else {
                        debug!("add node: id={}, address={}", id, address.index_address);
                        self.addresses
                            .insert(id.clone(), address.index_address.clone());
                        self.clients.insert(
                            id.clone(),
                            Arc::new(create_client(address.index_address.clone())),
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
                    self.leader_id = reply.get_leader_id();
                    return Ok(());
                }
                State::WRONG_LEADER => {
                    self.leader_id = reply.get_leader_id();
                    cnt_retry += 1;
                    debug!("retry with a new leader: id={}", self.leader_id);
                    continue;
                }
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "failed to delete documents in bulk",
                    ));
                }
            };
        }
    }

    pub fn commit(&mut self) -> Result<(), std::io::Error> {
        let mut req = CommitReq::new();
        req.set_client_id(self.client_id);

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

            let reply = match client.commit(&req) {
                Ok(r) => r,
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "failed to request committing index",
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
                        if grpc_address == address.index_address.as_str() {
                            debug!(
                                "node has not been changed: id={}, address={}",
                                id, grpc_address
                            );
                        } else {
                            debug!("update node: id={}, address={}", id, address.index_address);
                            self.addresses
                                .insert(id.clone(), address.index_address.clone());
                            self.clients.insert(
                                id.clone(),
                                Arc::new(create_client(address.index_address.clone())),
                            );
                        }
                    } else {
                        debug!("add node: id={}, address={}", id, address.index_address);
                        self.addresses
                            .insert(id.clone(), address.index_address.clone());
                        self.clients.insert(
                            id.clone(),
                            Arc::new(create_client(address.index_address.clone())),
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
                    self.leader_id = reply.get_leader_id();
                    return Ok(());
                }
                State::WRONG_LEADER => {
                    self.leader_id = reply.get_leader_id();
                    cnt_retry += 1;
                    debug!("retry with a new leader: id={}", self.leader_id);
                    continue;
                }
                _ => {
                    return Err(Error::new(ErrorKind::Other, "failed to commit index"));
                }
            };
        }
    }

    pub fn rollback(&mut self) -> Result<(), std::io::Error> {
        let mut req = RollbackReq::new();
        req.set_client_id(self.client_id);

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

            let reply = match client.rollback(&req) {
                Ok(r) => r,
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "failed to request rolling back index",
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
                        if grpc_address == address.index_address.as_str() {
                            debug!(
                                "node has not been changed: id={}, address={}",
                                id, grpc_address
                            );
                        } else {
                            debug!("update node: id={}, address={}", id, address.index_address);
                            self.addresses
                                .insert(id.clone(), address.index_address.clone());
                            self.clients.insert(
                                id.clone(),
                                Arc::new(create_client(address.index_address.clone())),
                            );
                        }
                    } else {
                        debug!("add node: id={}, address={}", id, address.index_address);
                        self.addresses
                            .insert(id.clone(), address.index_address.clone());
                        self.clients.insert(
                            id.clone(),
                            Arc::new(create_client(address.index_address.clone())),
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
                    self.leader_id = reply.get_leader_id();
                    return Ok(());
                }
                State::WRONG_LEADER => {
                    self.leader_id = reply.get_leader_id();
                    cnt_retry += 1;
                    debug!("retry with a new leader: id={}", self.leader_id);
                    continue;
                }
                _ => {
                    return Err(Error::new(ErrorKind::Other, "failed to rollback index"));
                }
            };
        }
    }

    pub fn merge(&mut self) -> Result<(), std::io::Error> {
        let mut req = MergeReq::new();
        req.set_client_id(self.client_id);

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

            let reply = match client.merge(&req) {
                Ok(r) => r,
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "failed to request merging index",
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
                        if grpc_address == address.index_address.as_str() {
                            debug!(
                                "node has not been changed: id={}, address={}",
                                id, grpc_address
                            );
                        } else {
                            debug!("update node: id={}, address={}", id, address.index_address);
                            self.addresses
                                .insert(id.clone(), address.index_address.clone());
                            self.clients.insert(
                                id.clone(),
                                Arc::new(create_client(address.index_address.clone())),
                            );
                        }
                    } else {
                        debug!("add node: id={}, address={}", id, address.index_address);
                        self.addresses
                            .insert(id.clone(), address.index_address.clone());
                        self.clients.insert(
                            id.clone(),
                            Arc::new(create_client(address.index_address.clone())),
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
                    self.leader_id = reply.get_leader_id();
                    return Ok(());
                }
                State::WRONG_LEADER => {
                    self.leader_id = reply.get_leader_id();
                    cnt_retry += 1;
                    debug!("retry with a new leader: id={}", self.leader_id);
                    continue;
                }
                _ => {
                    return Err(Error::new(ErrorKind::Other, "failed to merge index"));
                }
            };
        }
    }

    pub fn schema(&mut self) -> Result<String, std::io::Error> {
        let mut req = SchemaReq::new();
        req.set_client_id(self.client_id);

        let max_retry = 10;
        let mut cnt_retry = 0;

        loop {
            if max_retry < cnt_retry {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("max retry count has been exceeded: max_retry={}", max_retry),
                ));
            }

            let client = match self.clients.get(&self.node_id) {
                Some(c) => c,
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("failed to get client for node: id={}", self.node_id),
                    ));
                }
            };

            let reply = match client.schema(&req) {
                Ok(r) => r,
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "failed to request getting schema",
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
                        if grpc_address == address.index_address.as_str() {
                            debug!(
                                "node has not been changed: id={}, address={}",
                                id, grpc_address
                            );
                        } else {
                            debug!("update node: id={}, address={}", id, address.index_address);
                            self.addresses
                                .insert(id.clone(), address.index_address.clone());
                            self.clients.insert(
                                id.clone(),
                                Arc::new(create_client(address.index_address.clone())),
                            );
                        }
                    } else {
                        debug!("add node: id={}, address={}", id, address.index_address);
                        self.addresses
                            .insert(id.clone(), address.index_address.clone());
                        self.clients.insert(
                            id.clone(),
                            Arc::new(create_client(address.index_address.clone())),
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

            // continue to use the same node.
            for (id, address) in &self.addresses.clone() {
                if address == self.address.as_str() {
                    self.node_id = *id;
                    break;
                }
            }

            match reply.get_state() {
                State::OK => {
                    self.leader_id = reply.get_leader_id();
                    return Ok(String::from(reply.get_schema()));
                }
                _ => {
                    cnt_retry += 1;
                    warn!("failed to get schema");
                }
            }
        }
    }

    pub fn status(&mut self) -> Result<String, std::io::Error> {
        let mut req = StatusReq::new();
        req.set_client_id(self.client_id);

        let max_retry = 10;
        let mut cnt_retry = 0;

        loop {
            if max_retry < cnt_retry {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("max retry count has been exceeded: max_retry={}", max_retry),
                ));
            }

            let client = match self.clients.get(&self.node_id) {
                Some(c) => c,
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("failed to get client for node: id={}", self.node_id),
                    ));
                }
            };

            let reply = match client.status(&req) {
                Ok(r) => r,
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "failed to request getting status",
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
                        if grpc_address == address.index_address.as_str() {
                            debug!(
                                "node has not been changed: id={}, address={}",
                                id, grpc_address
                            );
                        } else {
                            debug!("update node: id={}, address={}", id, address.index_address);
                            self.addresses
                                .insert(id.clone(), address.index_address.clone());
                            self.clients.insert(
                                id.clone(),
                                Arc::new(create_client(address.index_address.clone())),
                            );
                        }
                    } else {
                        debug!("add node: id={}, address={}", id, address.index_address);
                        self.addresses
                            .insert(id.clone(), address.index_address.clone());
                        self.clients.insert(
                            id.clone(),
                            Arc::new(create_client(address.index_address.clone())),
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

            // continue to use the same node.
            for (id, address) in &self.addresses.clone() {
                if address == self.address.as_str() {
                    self.node_id = *id;
                    break;
                }
            }

            match reply.get_state() {
                State::OK => {
                    self.leader_id = reply.get_leader_id();

                    let address_map: HashMap<u64, NodeAddress> =
                        deserialize(&reply.get_address_map()).unwrap();

                    let mut nodes: Value = json!([]);
                    for (id, address) in address_map {
                        let mut node: Value = json!({});
                        node.as_object_mut()
                            .unwrap()
                            .insert("id".to_string(), serde_json::to_value(&id).unwrap());
                        node.as_object_mut().unwrap().insert(
                            "address".to_string(),
                            serde_json::to_value(&address).unwrap(),
                        );
                        nodes.as_array_mut().unwrap().push(node);
                    }

                    let mut v: Value = json!({});
                    v.as_object_mut().unwrap().insert(
                        "status".to_string(),
                        serde_json::to_value(&reply.get_status()).unwrap(),
                    );
                    v.as_object_mut().unwrap().insert(
                        "leader".to_string(),
                        serde_json::to_value(&self.leader_id).unwrap(),
                    );
                    v.as_object_mut()
                        .unwrap()
                        .insert("nodes".to_string(), nodes);

                    return Ok(v.to_string());
                }
                _ => {
                    cnt_retry += 1;
                    warn!("failed to get status");
                }
            }
        }
    }
}
