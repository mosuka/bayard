use clap::ArgMatches;
use serde_json::Value;

use bayard_client::index::client::IndexClient;

use crate::log::set_logger;

pub fn run_set_cli(matches: &ArgMatches) -> Result<(), std::io::Error> {
    set_logger();

    let server = matches.value_of("SERVER").unwrap();
    let id = matches.value_of("ID").unwrap();
    let fields = matches.value_of("FIELDS").unwrap();

    let mut doc_json: Value = serde_json::from_str(fields).unwrap();
    doc_json["_id"] = Value::String(id.to_string());
    let doc = serde_json::to_string(&doc_json).unwrap();

    let mut index_client = IndexClient::new(server);

    index_client.set(doc)
}
