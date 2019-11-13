use std::collections::HashMap;

use clap::ArgMatches;
use log::*;
use raft::eraftpb::{ConfChange, ConfChangeType};

use crate::client::client::create_client;
use crate::proto::indexrpcpb::ConfChangeReq;
use crate::server::util::conf_change;
use crate::util::log::set_log_level;

pub fn run_leave_cli(matches: &ArgMatches) -> Result<(), String> {
    set_log_level();
    env_logger::init();

    let host = matches.value_of("HOST").unwrap();
    let port = matches.value_of("PORT").unwrap().parse::<u16>().unwrap();
    let id = matches.value_of("ID").unwrap().parse::<u64>().unwrap();
    let mut peers = HashMap::new();
    if let Some(peers_vec) = matches.values_of("PEERS") {
        peers_vec
            .map(|s| {
                let mut parts = s.split('=');
                let id = parts.next().unwrap().parse::<u64>().unwrap();
                let addr = parts.next().unwrap();
                peers.insert(id, create_client(addr));
            })
            .count();
    }
    if let Some(leader_id_str) = matches.value_of("LEADER_ID") {
        let leader_id = leader_id_str.parse::<u64>().unwrap();
        let mut cc = ConfChange::new();
        cc.set_id(id);
        cc.set_node_id(id);
        cc.set_change_type(ConfChangeType::RemoveNode);
        let mut req = ConfChangeReq::new();
        req.set_cc(cc);
        req.set_ip(host.to_owned());
        req.set_port(port as u32);
        conf_change(id, leader_id, &peers, req);
        info!("the node was successfully removed from the cluster");
    }

    Ok(())
}
