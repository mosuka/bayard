use std::fs;

use clap::ArgMatches;

use crate::client::client::{create_client, Clerk};
use crate::util::log::set_logger;

pub fn run_delete_cli(matches: &ArgMatches) -> Result<(), String> {
    set_logger();

    let servers: Vec<_> = matches
        .values_of("SERVERS")
        .unwrap()
        .map(|addr| create_client(addr))
        .collect();

    let client_id = rand::random();

    let mut client = Clerk::new(&servers, client_id);

    if matches.is_present("BULK") {
        let file = matches.value_of("FILE").unwrap();
        let docs = fs::read_to_string(file).unwrap();

        print!("{}", client.bulk_delete(&docs));
    } else {
        let doc_id = matches.value_of("ID").unwrap();

        print!("{}", client.delete(doc_id));
    }

    Ok(())
}
