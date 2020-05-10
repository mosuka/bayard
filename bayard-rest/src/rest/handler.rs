use std::io::Read;

use iron::headers::ContentType;
use iron::prelude::*;
use iron::{status, IronError, IronResult, Request, Response};
use persistent::Write;
use router::Router;
use serde_json::Value;
use urlencoded::UrlEncodedQuery;

use crate::rest::Client;

pub fn get(req: &mut Request) -> IronResult<Response> {
    let id = req
        .extensions
        .get::<Router>()
        .unwrap()
        .find("id")
        .unwrap_or("")
        .to_owned();

    let client = req.get::<Write<Client>>().unwrap();
    let mut index_client = client.lock().unwrap();
    match index_client.get(id) {
        Ok(s) => Ok(Response::with((ContentType::json().0, status::Ok, s))),
        Err(e) => Err(IronError::new(
            e,
            (status::InternalServerError, "failed to get document"),
        )),
    }
}

pub fn set(req: &mut Request) -> IronResult<Response> {
    let id = req
        .extensions
        .get::<Router>()
        .unwrap()
        .find("id")
        .unwrap_or("")
        .to_owned();
    let mut body = String::new();
    req.body
        .read_to_string(&mut body)
        .expect("Failed to read line");

    let mut doc_json: Value = serde_json::from_str(body.as_str()).unwrap();
    doc_json["_id"] = Value::String(id);
    let doc = serde_json::to_string(&doc_json).unwrap();

    let client = req.get::<Write<Client>>().unwrap();
    let mut index_client = client.lock().unwrap();
    match index_client.set(doc) {
        Ok(()) => Ok(Response::with((ContentType::json().0, status::Ok, ""))),
        Err(e) => Err(IronError::new(
            e,
            (status::InternalServerError, "failed to set document"),
        )),
    }
}

pub fn delete(req: &mut Request) -> IronResult<Response> {
    let id = req
        .extensions
        .get::<Router>()
        .unwrap()
        .find("id")
        .unwrap_or("")
        .to_owned();

    let client = req.get::<Write<Client>>().unwrap();
    let mut index_client = client.lock().unwrap();
    match index_client.delete(id) {
        Ok(()) => Ok(Response::with((ContentType::json().0, status::Ok, ""))),
        Err(e) => Err(IronError::new(
            e,
            (status::InternalServerError, "failed to delete document"),
        )),
    }
}

pub fn bulk_set(req: &mut Request) -> IronResult<Response> {
    let mut docs = String::new();
    req.body
        .read_to_string(&mut docs)
        .expect("Failed to read line");

    let client = req.get::<Write<Client>>().unwrap();
    let mut index_client = client.lock().unwrap();
    match index_client.bulk_set(docs) {
        Ok(()) => Ok(Response::with((ContentType::json().0, status::Ok, ""))),
        Err(e) => Err(IronError::new(
            e,
            (
                status::InternalServerError,
                "failed to set documents in bulk",
            ),
        )),
    }
}

pub fn bulk_delete(req: &mut Request) -> IronResult<Response> {
    let mut docs = String::new();
    req.body
        .read_to_string(&mut docs)
        .expect("Failed to read line");

    let client = req.get::<Write<Client>>().unwrap();
    let mut index_client = client.lock().unwrap();
    match index_client.bulk_delete(docs) {
        Ok(()) => Ok(Response::with((ContentType::json().0, status::Ok, ""))),
        Err(e) => Err(IronError::new(
            e,
            (
                status::InternalServerError,
                "failed to delete documents in bulk",
            ),
        )),
    }
}

pub fn commit(req: &mut Request) -> IronResult<Response> {
    let client = req.get::<Write<Client>>().unwrap();
    let mut index_client = client.lock().unwrap();
    match index_client.commit() {
        Ok(()) => Ok(Response::with((ContentType::plaintext().0, status::Ok, ""))),
        Err(e) => Err(IronError::new(
            e,
            (status::InternalServerError, "failed to commit index"),
        )),
    }
}

pub fn rollback(req: &mut Request) -> IronResult<Response> {
    let client = req.get::<Write<Client>>().unwrap();
    let mut index_client = client.lock().unwrap();
    match index_client.rollback() {
        Ok(()) => Ok(Response::with((ContentType::plaintext().0, status::Ok, ""))),
        Err(e) => Err(IronError::new(
            e,
            (status::InternalServerError, "failed to rollback index"),
        )),
    }
}

pub fn merge(req: &mut Request) -> IronResult<Response> {
    let client = req.get::<Write<Client>>().unwrap();
    let mut index_client = client.lock().unwrap();
    match index_client.merge() {
        Ok(()) => Ok(Response::with((ContentType::plaintext().0, status::Ok, ""))),
        Err(e) => Err(IronError::new(
            e,
            (status::InternalServerError, "failed to merge index"),
        )),
    }
}

pub fn schema(req: &mut Request) -> IronResult<Response> {
    let client = req.get::<Write<Client>>().unwrap();
    let mut index_client = client.lock().unwrap();
    match index_client.schema() {
        Ok(s) => Ok(Response::with((ContentType::json().0, status::Ok, s))),
        Err(e) => Err(IronError::new(
            e,
            (status::InternalServerError, "failed to get schema"),
        )),
    }
}

pub fn search(req: &mut Request) -> IronResult<Response> {
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
    let exclude_docs = map.contains_key("exclude_docs");
    let mut facet_field: &str = "";
    if map.contains_key("facet_field") {
        facet_field = map.get("facet_field").unwrap().get(0).unwrap();
    }
    let mut facet_prefixes = Vec::new();
    if map.contains_key("facet_prefix") {
        facet_prefixes = map.get("facet_prefix").cloned().unwrap();
    }

    let client = req.get::<Write<Client>>().unwrap();
    let mut index_client = client.lock().unwrap();
    match index_client.search(
        query,
        from,
        limit,
        exclude_count,
        exclude_docs,
        facet_field,
        facet_prefixes,
    ) {
        Ok(s) => Ok(Response::with((ContentType::json().0, status::Ok, s))),
        Err(e) => Err(IronError::new(
            e,
            (status::InternalServerError, "failed to search index"),
        )),
    }
}

pub fn status(req: &mut Request) -> IronResult<Response> {
    let client = req.get::<Write<Client>>().unwrap();
    let mut index_client = client.lock().unwrap();
    match index_client.status() {
        Ok(s) => Ok(Response::with((ContentType::json().0, status::Ok, s))),
        Err(e) => Err(IronError::new(
            e,
            (status::InternalServerError, "failed to get status"),
        )),
    }
}
