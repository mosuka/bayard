use std::io::Read;

use iron::headers::ContentType;
use iron::prelude::*;
use iron::{status, IronError, IronResult, Request, Response};
use persistent::Write;
use router::Router;
use serde_json::Value;

use crate::handler::Client;

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
