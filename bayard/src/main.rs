#[macro_use]
extern crate clap;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::io;
use std::net::ToSocketAddrs;
use std::path::Path;
use std::sync::Arc;

use bayard_proto::proto::{indexpb_grpc, raftpb_grpc};
use clap::{App, AppSettings, Arg};
use futures::Future;
use grpcio::{Environment, ServerBuilder};
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use log::*;
use raft::storage::MemStorage;

use bayard_client::raft::client::RaftClient;
use bayard_common::log::set_logger;
use bayard_common::signal::shutdown_signal;
use bayard_server::index::server::IndexServer;
use bayard_server::metric::handler::handle;
use bayard_server::raft::config::NodeAddress;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    set_logger();

    let threads = format!("{}", num_cpus::get().to_owned());

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
                .help("Number of indexer threads. By default indexer uses number of available logical cpu as threads count.")
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

    let id = matches
        .value_of("ID")
        .expect("ID was not set")
        .parse::<u64>()
        .expect("ID must be an integer");
    let host = matches.value_of("HOST").expect("HOST was not set");

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
    let mut peer_address = "".to_string();
    if let Some(_peer_address) = matches.value_of("PEER_RAFT_ADDRESS") {
        peer_address = _peer_address
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap()
            .to_string();
    }
    let data_directory = matches
        .value_of("DATA_DIRECTORY")
        .expect("DATA_DIRECTORY was not set");
    let schema_file = matches
        .value_of("SCHEMA_FILE")
        .expect("SCHEMA_FILE was not set");
    let tokenizer_file = matches
        .value_of("TOKENIZER_FILE")
        .expect("TOKENIZER_FILE was not set");
    let indexer_threads = matches
        .value_of("INDEXER_THREADS")
        .expect("INDEXER_THREADS was not set")
        .parse::<usize>()
        .expect("INDEXER_THREADS must be an integer");
    let indexer_memory_size = matches
        .value_of("INDEXER_MEMORY_SIZE")
        .expect("INDEXER_MEMORY_SIZE was not set")
        .parse::<usize>()
        .expect("INDEXER_MEMORY_SIZE must be an integer");

    let raft_address = format!("{}:{}", host, raft_port)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap()
        .to_string();
    let index_address = format!("{}:{}", host, index_port)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap()
        .to_string();

    let node_address = NodeAddress {
        index_address: index_address.clone(),
        raft_address: raft_address.clone(),
    };

    let mut addresses = HashMap::new();

    // change config
    if !peer_address.is_empty() {
        let mut client = RaftClient::new(&peer_address);
        match client.join(id, node_address.clone()) {
            Ok(_addresses) => addresses = _addresses,
            Err(e) => return Err(Box::try_from(e).unwrap()),
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
        .bind(index_address.split(":").next().unwrap(), index_port)
        .build()
        .unwrap();
    let mut raft_server = ServerBuilder::new(env_raft)
        .register_service(raft_service)
        .bind(raft_address.split(":").next().unwrap(), raft_port)
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
    let metrics_address = format!("{}:{}", host, metrics_port)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();
    let metrics_service = make_service_fn(|_| async { Ok::<_, io::Error>(service_fn(handle)) });
    let metrics_server = Server::bind(&metrics_address).serve(metrics_service);
    let metrics_server_graceful = metrics_server.with_graceful_shutdown(shutdown_signal());
    info!("start metrics service on {}:{}", host, metrics_port);

    match metrics_server_graceful.await {
        Ok(_) => {
            info!("stop metrics service on {}:{}", host, metrics_port);
        }
        Err(e) => error!(
            "failed to stop metrics service on {}:{}: error={}",
            host, metrics_port, e
        ),
    }

    match index_server.shutdown().wait() {
        Ok(_) => {
            info!("stop index service on {}:{}", host, index_port);
        }
        Err(e) => error!(
            "failed to stop index service on {}:{}: error={}",
            host, index_port, e
        ),
    }
    match raft_server.shutdown().wait() {
        Ok(_) => {
            info!("stop raft service on {}:{}", host, raft_port);
        }
        Err(e) => error!(
            "failed to stop raft service on {}:{}: error={}",
            host, raft_port, e
        ),
    }

    return Ok(());
}
