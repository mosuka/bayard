use std::collections::HashMap;

use clap::ArgMatches;

use crate::server::server::IndexServer;
use crate::util::log::set_logger;

pub fn run_serve_cli(matches: &ArgMatches) -> Result<(), String> {
    set_logger();

    let id = matches.value_of("ID").unwrap().parse::<u64>().unwrap();
    let host = matches.value_of("HOST").unwrap();
    let port = matches.value_of("PORT").unwrap().parse::<u16>().unwrap();
    let mut peers_addr = HashMap::new();
    if let Some(peers_addr_vec) = matches.values_of("PEERS") {
        peers_addr_vec
            .map(|s| {
                let mut parts = s.split('=');
                let peer_id = parts.next().unwrap().parse::<u64>().unwrap();
                let peer_addr = parts.next().unwrap();
                peers_addr.insert(peer_id, peer_addr.to_string());
            })
            .count();
    }
    let data_directory = matches.value_of("DATA_DIRECTORY").unwrap();
    let schema_file = matches.value_of("SCHEMA_FILE").unwrap();

    IndexServer::start_server(id, host, port, peers_addr, data_directory, schema_file);

    Ok(())
}
