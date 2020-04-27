use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use clap::ArgMatches;
use crossbeam_channel::select;
use futures::Future;
use grpcio::{Environment, ServerBuilder};
use log::*;
use raft::storage::MemStorage;

use bayard_client::raft::client::RaftClient;
use bayard_proto::proto::{indexpb_grpc, raftpb_grpc};
use bayard_server::index::server::IndexServer;
use bayard_server::raft::config::NodeAddress;

use crate::log::set_logger;
use crate::signal::sigterm_channel;
// use bayard_server::metrics::Metrics;

pub fn run_start_cli(matches: &ArgMatches) -> Result<(), std::io::Error> {
    set_logger();

    let id = matches.value_of("ID").unwrap().parse::<u64>().unwrap();
    let host = matches.value_of("HOST").unwrap();
    let raft_port = matches
        .value_of("RAFT_PORT")
        .unwrap()
        .parse::<u16>()
        .unwrap();
    let index_port = matches
        .value_of("INDEX_PORT")
        .unwrap()
        .parse::<u16>()
        .unwrap();
    let mut peer_address = "";
    if let Some(_peer_address) = matches.value_of("PEER_RAFT_ADDRESS") {
        peer_address = _peer_address;
    }
    let data_directory = matches.value_of("DATA_DIRECTORY").unwrap();
    let schema_file = matches.value_of("SCHEMA_FILE").unwrap();
    let tokenizer_file = matches.value_of("TOKENIZER_FILE").unwrap();
    let indexer_threads = matches
        .value_of("INDEXER_THREADS")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let indexer_memory_size = matches
        .value_of("INDEXER_MEMORY_SIZE")
        .unwrap()
        .parse::<usize>()
        .unwrap();

    let raft_address = format!("{}:{}", host, raft_port);
    let index_address = format!("{}:{}", host, index_port);

    let node_address = NodeAddress {
        index_address,
        raft_address,
    };

    let mut addresses = HashMap::new();

    // change config
    if peer_address != "" {
        let mut client = RaftClient::new(peer_address);
        match client.join(id, node_address.clone()) {
            Ok(_addresses) => addresses = _addresses,
            Err(e) => return Err(e),
        };
    }

    let env_index = Arc::new(Environment::new(10));
    let env_raft = Arc::new(Environment::new(10));

    let index_path = Path::new(data_directory)
        .join("index")
        .to_str()
        .unwrap()
        .to_string();
    let raft_storage = MemStorage::new();

    // let metrics = Metrics::new(id);

    let (index, raft) = IndexServer::new(
        index_path,
        schema_file,
        tokenizer_file,
        indexer_threads,
        indexer_memory_size,
        raft_storage,
        id,
        node_address,
        addresses,
        // metrics,
    );

    let index_service = indexpb_grpc::create_index_service(index);
    let raft_service = raftpb_grpc::create_raft_service(raft);

    let mut index_server = ServerBuilder::new(env_index)
        .register_service(index_service)
        .bind(host, index_port)
        .build()
        .unwrap();
    let mut raft_server = ServerBuilder::new(env_raft)
        .register_service(raft_service)
        .bind(host, raft_port)
        .build()
        .unwrap();

    index_server.start();
    raft_server.start();

    for &(ref h, p) in index_server.bind_addrs() {
        info!("start index service on {}:{}", h, p);
    }

    for &(ref h, p) in raft_server.bind_addrs() {
        info!("start Raft service on {}:{}", h, p);
    }

    // Wait for signals for termination (SIGINT, SIGTERM).
    let sigterm_receiver = sigterm_channel().unwrap();
    loop {
        select! {
            recv(sigterm_receiver) -> _ => {
                debug!("receive signal");
                break;
            }
        }
    }

    match index_server.shutdown().wait() {
        Ok(_) => {
            info!("stop index service on {}:{}", host, index_port);
        }
        Err(e) => error!("{}", e),
    }
    match raft_server.shutdown().wait() {
        Ok(_) => {
            info!("stop Raft service on {}:{}", host, raft_port);
        }
        Err(e) => error!("{}", e),
    }

    Ok(())
}
