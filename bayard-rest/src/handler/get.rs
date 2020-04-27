use iron::headers::ContentType;
use iron::prelude::*;
use iron::{status, IronError, IronResult, Request, Response};
use persistent::Write;
use router::Router;

use crate::handler::Client;

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
