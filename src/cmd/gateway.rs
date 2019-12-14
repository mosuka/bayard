use std::io::Read;

use clap::ArgMatches;
use iron::{Chain, Iron, IronResult, Request, Response, status};
use iron::headers::ContentType;
use iron::prelude::*;
use iron::typemap::Key;
use logger::Logger;
use persistent::Write;
use router::Router;
use urlencoded::UrlEncodedQuery;

use crate::client::client::{Clerk, create_client};
use crate::util::log::set_http_logger;

#[derive(Copy, Clone)]
pub struct Client;

impl Key for Client {
    type Value = Clerk;
}

fn probe(req: &mut Request) -> IronResult<Response> {
    let client_arc = req.get::<Write<Client>>().unwrap();
    let mut client = client_arc.lock().unwrap();
    let value = client.probe();

    Ok(Response::with((
        ContentType::plaintext().0,
        status::Ok,
        value,
    )))
}

fn peers(req: &mut Request) -> IronResult<Response> {
    let client_arc = req.get::<Write<Client>>().unwrap();
    let mut client = client_arc.lock().unwrap();
    let value = client.peers();

    Ok(Response::with((ContentType::json().0, status::Ok, value)))
}

fn metrics(req: &mut Request) -> IronResult<Response> {
    let client_arc = req.get::<Write<Client>>().unwrap();
    let mut client = client_arc.lock().unwrap();
    let value = client.metrics();

    Ok(Response::with((
        ContentType::plaintext().0,
        status::Ok,
        value,
    )))
}

fn get(req: &mut Request) -> IronResult<Response> {
    let doc_id = req
        .extensions
        .get::<Router>()
        .unwrap()
        .find("doc_id")
        .unwrap_or("")
        .to_owned();

    let client_arc = req.get::<Write<Client>>().unwrap();
    let mut client = client_arc.lock().unwrap();
    let value = client.get(&doc_id);

    Ok(Response::with((ContentType::json().0, status::Ok, value)))
}

fn put(req: &mut Request) -> IronResult<Response> {
    let doc_id = req
        .extensions
        .get::<Router>()
        .unwrap()
        .find("doc_id")
        .unwrap_or("")
        .to_owned();
    let mut body = String::new();
    req.body
        .read_to_string(&mut body)
        .expect("Failed to read line");

    let client_arc = req.get::<Write<Client>>().unwrap();
    let mut client = client_arc.lock().unwrap();
    let value = client.put(&doc_id, &body);

    Ok(Response::with((ContentType::json().0, status::Ok, value)))
}

fn delete(req: &mut Request) -> IronResult<Response> {
    let doc_id = req
        .extensions
        .get::<Router>()
        .unwrap()
        .find("doc_id")
        .unwrap_or("")
        .to_owned();

    let client_arc = req.get::<Write<Client>>().unwrap();
    let mut client = client_arc.lock().unwrap();
    let value = client.delete(&doc_id);

    Ok(Response::with((ContentType::json().0, status::Ok, value)))
}

fn commit(req: &mut Request) -> IronResult<Response> {
    let client_arc = req.get::<Write<Client>>().unwrap();
    let mut client = client_arc.lock().unwrap();
    let value = client.commit();

    Ok(Response::with((ContentType::json().0, status::Ok, value)))
}

fn rollback(req: &mut Request) -> IronResult<Response> {
    let client_arc = req.get::<Write<Client>>().unwrap();
    let mut client = client_arc.lock().unwrap();
    let value = client.rollback();

    Ok(Response::with((ContentType::json().0, status::Ok, value)))
}

fn merge(req: &mut Request) -> IronResult<Response> {
    let client_arc = req.get::<Write<Client>>().unwrap();
    let mut client = client_arc.lock().unwrap();
    let value = client.merge();

    Ok(Response::with((ContentType::json().0, status::Ok, value)))
}

fn search(req: &mut Request) -> IronResult<Response> {
    let map = req.get_ref::<UrlEncodedQuery>().unwrap().to_owned();
    let query = map.get("query").unwrap().get(0).unwrap();

    let mut from = 0;
    if map.contains_key("from") {
        from = map
            .get("from")
            .unwrap()
            .get(0)
            .unwrap_or(&String::from("0"))
            .parse::<u64>()
            .unwrap();
    }
    let mut limit = 10;
    if map.contains_key("limit") {
        limit = map
            .get("limit")
            .unwrap()
            .get(0)
            .unwrap_or(&String::from("10"))
            .parse::<u64>()
            .unwrap();
    }
    let exclude_count = map.contains_key("exclude_count");
//    if map.contains_key("exclude_count") {
//        exclude_count = map
//            .get("exclude_count")
//            .unwrap()
//            .get(0)
//            .unwrap_or(&String::from("true"))
//            .parse::<bool>()
//            .unwrap();
//    }
    let exclude_docs = map.contains_key("exclude_docs");
//    if map.contains_key("exclude_docs") {
//        exclude_docs = map
//            .get("exclude_docs")
//            .unwrap()
//            .get(0)
//            .unwrap_or(&String::from("true"))
//            .parse::<bool>()
//            .unwrap();
//    }
    let mut facet_field: &str = "";
    if map.contains_key("facet_field") {
        facet_field = map
            .get("facet_field")
            .unwrap()
            .get(0)
            .unwrap();
    }
    let mut facet_prefixes = Vec::new();
    if map.contains_key("facet_prefix") {
        facet_prefixes = map
            .get("facet_prefix").cloned().unwrap();
    }


    let client_arc = req.get::<Write<Client>>().unwrap();
    let mut client = client_arc.lock().unwrap();
    let value = client.search(query, from, limit, exclude_count, exclude_docs, facet_field, facet_prefixes);

    Ok(Response::with((ContentType::json().0, status::Ok, value)))
}

fn schema(req: &mut Request) -> IronResult<Response> {
    let client_arc = req.get::<Write<Client>>().unwrap();
    let mut client = client_arc.lock().unwrap();
    let value = client.schema();

    Ok(Response::with((ContentType::json().0, status::Ok, value)))
}

pub fn run_gateway_cli(matches: &ArgMatches) -> Result<(), String> {
    set_http_logger();

    let host = matches.value_of("HOST").unwrap();
    let port = matches.value_of("PORT").unwrap().parse::<u16>().unwrap();
    let servers: Vec<_> = matches
        .values_of("SERVERS")
        .unwrap()
        .map(|addr| create_client(addr))
        .collect();

    let addr = format!("{}:{}", host, port);

    let client_id: u64 = rand::random();

    let client = Clerk::new(&servers, client_id);

    let (logger_before, logger_after) = Logger::new(None);

    let mut router = Router::new();
    router.get("/probe", probe, "probe");
    router.get("/peers", peers, "peers");
    router.get("/metrics", metrics, "metrics");
    router.get("/index/docs/:doc_id", get, "get");
    router.put("/index/docs/:doc_id", put, "put");
    router.delete("/index/docs/:doc_id", delete, "delete");
    router.get("/index/search", search, "search");
    router.get("/index/commit", commit, "commit");
    router.get("/index/rollback", rollback, "rollback");
    router.get("/index/merge", merge, "merge");
    router.get("/index/schema", schema, "schema");

    let mut chain = Chain::new(router);
    chain.link_before(logger_before);
    chain.link(Write::<Client>::both(client));
    chain.link_after(logger_after);

    Iron::new(chain).http(addr).unwrap();

    Ok(())
}
