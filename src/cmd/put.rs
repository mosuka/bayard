use std::fs;

use bayard_client::client::client::{create_client, Clerk};
use clap::ArgMatches;
use serde_json;
use serde_json::Value;

use crate::util::log::set_logger;

pub fn run_put_cli(matches: &ArgMatches) -> Result<(), String> {
    set_logger();

    let servers: Vec<_> = matches
        .values_of("SERVERS")
        .unwrap()
        .map(|addr| create_client(addr))
        .collect();

    let client_id = rand::random();

    let mut client = Clerk::new(&servers, client_id);

    let file = matches.value_of("FILE").unwrap();

    if matches.is_present("BULK") {
        let docs = fs::read_to_string(file).unwrap();

        print!("{}", client.bulk_put(&docs));
    } else {
        let mut doc = fs::read_to_string(file).unwrap();

        if let Some(_doc_id) = matches.value_of("ID") {
            let mut doc_json: Value = serde_json::from_str(doc.as_str()).unwrap();
            doc_json["_id"] = Value::String(_doc_id.to_string());
            doc = serde_json::to_string(&doc_json).unwrap();
        }

        print!("{}", client.put(&doc));
    }

    Ok(())
}
