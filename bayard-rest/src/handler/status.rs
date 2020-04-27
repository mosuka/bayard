use iron::headers::ContentType;
use iron::prelude::*;
use iron::{status, IronError, IronResult, Request, Response};
use persistent::Write;

use crate::handler::Client;

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
