#[macro_use]
extern crate clap;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use bayard_proto::proto::{indexpb_grpc, raftpb_grpc};
use clap::{App, AppSettings, Arg};
use crossbeam_channel::select;
use futures::Future;
use grpcio::{Environment, ServerBuilder};
use iron::{Chain, Iron};
use log::*;
use logger::Logger;
use raft::storage::MemStorage;
use router::Router;

use bayard_client::raft::client::RaftClient;
use bayard_common::log::set_logger;
use bayard_common::signal::sigterm_channel;
use bayard_server::index::server::IndexServer;
use bayard_server::metric::handler::metrics;
use bayard_server::raft::config::NodeAddress;

fn main() -> Result<(), std::io::Error> {
    set_logger();

    let cpus = num_cpus::get().to_owned();
    let threads;
    if cpus > 1 {
        threads = format!("{}", cpus - 1);
    } else {
        threads = format!("{}", cpus);
    }

    let app = App::new(crate_name!())
        .setting(AppSettings::DeriveDisplayOrder)
        .version(crate_version!())
        .author(crate_authors!())
        .about("Bayard server")
        .help_message("Prints help information.")
        .version_message("Prints version information.")
        .version_short("v")
        .arg(
            Arg::with_name("ID")
                .help("Node ID.")
                .value_name("ID")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("HOST")
                .help("Node address.")
                .short("H")
                .long("host")
                .value_name("HOST")
                .default_value("0.0.0.0")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("RAFT_PORT")
                .help("Raft service port number.")
                .short("r")
                .long("raft-port")
                .value_name("RAFT_PORT")
                .default_value("7000")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("INDEX_PORT")
                .help("Index service port number")
                .short("i")
                .long("index-port")
                .value_name("INDEX_PORT")
                .default_value("5000")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("METRICS_PORT")
                .help("Metrics service port number")
                .short("M")
                .long("metrics-port")
                .value_name("METRICS_PORT")
                .default_value("9000")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("PEER_RAFT_ADDRESS")
                .help("Raft address of a peer node running in an existing cluster.")
                .short("p")
                .long("peer-raft-address")
                .value_name("IP:PORT")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("DATA_DIRECTORY")
                .help("Data directory. Stores index, snapshots, and raft logs.")
                .short("d")
                .long("data-directory")
                .value_name("DATA_DIRECTORY")
                .default_value("./data")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("SCHEMA_FILE")
                .help("Schema file. Must specify An existing file name.")
                .short("s")
                .long("schema-file")
                .value_name("SCHEMA_FILE")
                .default_value("./etc/schema.json")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("TOKENIZER_FILE")
                .help("Tokenizer file. Must specify An existing file name.")
                .short("T")
                .long("tokenizer-file")
                .value_name("TOKENIZER_FILE")
                .default_value("./etc/tokenizer.json")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("INDEXER_THREADS")
                .help("Number of indexer threads.")
                .short("t")
                .long("indexer-threads")
                .value_name("INDEXER_THREADS")
                .default_value(&threads)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("INDEXER_MEMORY_SIZE")
                .help("Total memory size (in bytes) used by the indexer.")
                .short("m")
                .long("indexer-memory-size")
                .value_name("INDEXER_MEMORY_SIZE")
                .default_value("1000000000")
                .takes_value(true),
        );

    let matches = app.get_matches();

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
    let metrics_port = matches
        .value_of("METRICS_PORT")
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
    let metrics_address = format!("{}:{}", host, metrics_port);

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
            // Err(e) => return Err(Box::try_from(e).unwrap()),
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
        info!("start raft service on {}:{}", h, p);
    }

    // metrics service
    let (logger_before, logger_after) = Logger::new(None);
    let mut router = Router::new();
    router.get("/metrics", metrics, "metrics");

    let mut chain = Chain::new(router);
    chain.link_before(logger_before);
    chain.link_after(logger_after);

    let mut metrics_server = Iron::new(chain).http(metrics_address).unwrap();
    info!("start metrics service on {}:{}", host, metrics_port);

    // Wait for signals for termination (SIGINT, SIGTERM).
    let sigterm_receiver = sigterm_channel().unwrap();
    loop {
        select! {
            recv(sigterm_receiver) -> _ => {
                info!("receive signal");
                break;
            }
        }
    }

    match metrics_server.close() {
        Ok(_) => {
            info!("stop metrics service on {}:{}", host, metrics_port);
        }
        Err(e) => error!("{}", e),
    }

    match index_server.shutdown().wait() {
        Ok(_) => {
            info!("stop index service on {}:{}", host, index_port);
        }
        Err(e) => error!("{}", e),
    }
    match raft_server.shutdown().wait() {
        Ok(_) => {
            info!("stop raft service on {}:{}", host, raft_port);
        }
        Err(e) => error!("{}", e),
    }

    return Ok(());
}
