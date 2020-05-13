use clap::ArgMatches;
use serde_json::Value;

use bayard_client::index::client::IndexClient;
use bayard_client::raft::client::RaftClient;

pub fn leave(matches: &ArgMatches) -> Result<(), std::io::Error> {
    let server = matches.value_of("SERVER").unwrap();
    let id = matches.value_of("ID").unwrap().parse::<u64>().unwrap();

    let mut raft_client = RaftClient::new(server);

    match raft_client.leave(id) {
        Ok(v) => {
            println!("{}", serde_json::to_string(&v).unwrap());
            Ok(())
        }
        Err(e) => {
            println!("{}", e);
            Err(e)
        }
    }
}

pub fn status(matches: &ArgMatches) -> Result<(), std::io::Error> {
    let server = matches.value_of("SERVER").unwrap();

    let mut index_client = IndexClient::new(server);

    match index_client.status() {
        Ok(v) => {
            println!("{}", v);
            Ok(())
        }
        Err(e) => {
            println!("{}", e);
            Err(e)
        }
    }
}

pub fn schema(matches: &ArgMatches) -> Result<(), std::io::Error> {
    let server = matches.value_of("SERVER").unwrap();

    let mut index_client = IndexClient::new(server);

    match index_client.schema() {
        Ok(v) => {
            println!("{}", v);
            Ok(())
        }
        Err(e) => {
            println!("{}", e);
            Err(e)
        }
    }
}

pub fn get(matches: &ArgMatches) -> Result<(), std::io::Error> {
    let server = matches.value_of("SERVER").unwrap();
    let id = matches.value_of("ID").unwrap();

    let mut index_client = IndexClient::new(server);

    match index_client.get(id.to_string()) {
        Ok(v) => {
            println!("{}", v);
            Ok(())
        }
        Err(e) => {
            println!("{}", e);
            Err(e)
        }
    }
}

pub fn search(matches: &ArgMatches) -> Result<(), std::io::Error> {
    let server = matches.value_of("SERVER").unwrap();
    let from = matches.value_of("FROM").unwrap().parse::<u64>().unwrap();
    let limit = matches.value_of("LIMIT").unwrap().parse::<u64>().unwrap();
    let exclude_count = matches.is_present("EXCLUDE_COUNT");
    let exclude_docs = matches.is_present("EXCLUDE_DOCS");
    let facet_field = matches.value_of("FACET_FIELD").unwrap();
    let mut facet_prefixes: Vec<String> = Vec::new();
    if let Some(_facet_prefixes) = matches.values_of("FACET_PREFIX") {
        _facet_prefixes
            .map(|s| facet_prefixes.push(s.to_string()))
            .count();
    }
    let query = matches.value_of("QUERY").unwrap();

    let mut index_client = IndexClient::new(server);

    match index_client.search(
        query,
        from,
        limit,
        exclude_count,
        exclude_docs,
        facet_field,
        facet_prefixes,
    ) {
        Ok(v) => {
            println!("{}", v);
            Ok(())
        }
        Err(e) => {
            println!("{}", e);
            Err(e)
        }
    }
}

pub fn set(matches: &ArgMatches) -> Result<(), std::io::Error> {
    let server = matches.value_of("SERVER").unwrap();
    let id = matches.value_of("ID").unwrap();
    let fields = matches.value_of("FIELDS").unwrap();

    let mut doc_json: Value = serde_json::from_str(fields).unwrap();
    doc_json["_id"] = Value::String(id.to_string());
    let doc = serde_json::to_string(&doc_json).unwrap();

    let mut index_client = IndexClient::new(server);

    index_client.set(doc)
}

pub fn delete(matches: &ArgMatches) -> Result<(), std::io::Error> {
    let server = matches.value_of("SERVER").unwrap();
    let id = matches.value_of("ID").unwrap();

    let mut index_client = IndexClient::new(server);

    index_client.delete(id.to_string())
}

pub fn bulk_set(matches: &ArgMatches) -> Result<(), std::io::Error> {
    let server = matches.value_of("SERVER").unwrap();
    let docs = matches.value_of("DOCS").unwrap();

    let mut index_client = IndexClient::new(server);

    index_client.bulk_set(docs.to_string())
}

pub fn bulk_delete(matches: &ArgMatches) -> Result<(), std::io::Error> {
    let server = matches.value_of("SERVER").unwrap();
    let docs = matches.value_of("DOCS").unwrap();

    let mut index_client = IndexClient::new(server);

    index_client.bulk_delete(docs.to_string())
}

pub fn commit(matches: &ArgMatches) -> Result<(), std::io::Error> {
    let server = matches.value_of("SERVER").unwrap();

    let mut index_client = IndexClient::new(server);

    index_client.commit()
}

pub fn rollback(matches: &ArgMatches) -> Result<(), std::io::Error> {
    let server = matches.value_of("SERVER").unwrap();

    let mut index_client = IndexClient::new(server);

    index_client.rollback()
}

pub fn merge(matches: &ArgMatches) -> Result<(), std::io::Error> {
    let server = matches.value_of("SERVER").unwrap();

    let mut index_client = IndexClient::new(server);

    index_client.merge()
}
