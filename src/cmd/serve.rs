use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use clap::ArgMatches;
use log::*;
use serde_json;
use tantivy::schema::Schema;
use tantivy::Index;

use crate::client::client::{create_client, Clerk};
use crate::proto::indexpb_grpc::IndexClient;
use crate::server::server::IndexServer;
use crate::util::log::set_logger;

pub fn run_serve_cli(matches: &ArgMatches) -> Result<(), String> {
    set_logger();

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
    let leader_id = matches
        .value_of("LEADER_ID")
        .unwrap_or("0")
        .parse::<u64>()
        .unwrap();
    let data_directory = matches.value_of("DATA_DIRECTORY").unwrap();
    let schema_file = matches.value_of("SCHEMA_FILE").unwrap();
    let unique_key_field_name = matches.value_of("UNIQUE_KEY_FIELD_NAME").unwrap();

    let data_directory_path = Path::new(data_directory);
    fs::create_dir_all(&data_directory_path).unwrap_or_default();

    let raft_path = data_directory_path.join(Path::new("raft"));
    fs::create_dir_all(&raft_path).unwrap_or_default();

    let index_path = data_directory_path.join(Path::new("index"));
    let index = if index_path.exists() {
        Index::open_in_dir(index_path.to_str().unwrap()).unwrap()
    } else {
        let schema_content = fs::read_to_string(schema_file).unwrap();
        let schema: Schema =
            serde_json::from_str(&schema_content).expect("error while reading json");
        fs::create_dir_all(&index_path).unwrap_or_default();
        Index::create_in_dir(index_path.to_str().unwrap(), schema).unwrap()
    };

    info!("starting a server...");
    debug!("host: {}", host);
    debug!("port: {}", port);
    debug!("data_directory: {}", data_directory);
    debug!("schema_file: {}", schema_file);
    debug!("id: {}", id);

    IndexServer::start_server(
        id,
        host,
        port,
        leader_id,
        peers,
        Arc::new(index),
        unique_key_field_name,
    );

    Ok(())
}
