use std::io::Read;

use clap::ArgMatches;
use iron::headers::ContentType;
use iron::prelude::*;
use iron::typemap::Key;
use iron::{status, Chain, Iron, IronResult, Request, Response};
use logger::Logger;
use persistent::Write;
use router::Router;

use crate::client::client::{create_client, Clerk};
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
    client.put(&doc_id, &body);

    Ok(Response::with((ContentType::json().0, status::Ok, "")))
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
    client.delete(&doc_id);

    Ok(Response::with((ContentType::json().0, status::Ok, "")))
}

fn search(req: &mut Request) -> IronResult<Response> {
    let mut body = String::new();
    req.body
        .read_to_string(&mut body)
        .expect("Failed to read line");

    let client_arc = req.get::<Write<Client>>().unwrap();
    let mut client = client_arc.lock().unwrap();
    let value = client.search(&body);

    Ok(Response::with((ContentType::json().0, status::Ok, value)))
}

fn commit(req: &mut Request) -> IronResult<Response> {
    let client_arc = req.get::<Write<Client>>().unwrap();
    let mut client = client_arc.lock().unwrap();
    client.commit();

    Ok(Response::with((ContentType::json().0, status::Ok, "")))
}

fn rollback(req: &mut Request) -> IronResult<Response> {
    let client_arc = req.get::<Write<Client>>().unwrap();
    let mut client = client_arc.lock().unwrap();
    client.rollback();

    Ok(Response::with((ContentType::json().0, status::Ok, "")))
}

fn merge(req: &mut Request) -> IronResult<Response> {
    let client_arc = req.get::<Write<Client>>().unwrap();
    let mut client = client_arc.lock().unwrap();
    client.merge();

    Ok(Response::with((ContentType::json().0, status::Ok, "")))
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
    router.post("/index/search", search, "search");
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
