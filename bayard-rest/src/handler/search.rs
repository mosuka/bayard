use iron::headers::ContentType;
use iron::prelude::*;
use iron::{status, IronError, IronResult, Request, Response};
use persistent::Write;
use urlencoded::UrlEncodedQuery;

use crate::handler::Client;

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
