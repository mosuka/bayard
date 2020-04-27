use std::io::Read;

use iron::headers::ContentType;
use iron::prelude::*;
use iron::{status, IronError, IronResult, Request, Response};
use persistent::Write;

use crate::handler::Client;

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
