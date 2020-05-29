use hyper::{header, Body, Method, Request, Response, StatusCode};
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;

use bayard_client::index::client::IndexClient;
use lazy_static::lazy_static;

lazy_static! {
    static ref RE_DOCUMENTS: Regex = Regex::new(r"^/v1/documents/(?P<id>.+)$").unwrap();
    static ref RE_DOCUMENTS_BULK: Regex = Regex::new(r"^/v1/documents$").unwrap();
    static ref RE_COMMIT: Regex = Regex::new(r"^/v1/commit$").unwrap();
    static ref RE_ROLLBACK: Regex = Regex::new(r"^/v1/rollback$").unwrap();
    static ref RE_MERGE: Regex = Regex::new(r"^/v1/merge$").unwrap();
    static ref RE_SCHEMA: Regex = Regex::new(r"^/v1/schema$").unwrap();
    static ref RE_SEARCH: Regex = Regex::new(r"^/v1/search$").unwrap();
    static ref RE_STATUS: Regex = Regex::new(r"^/v1/status$").unwrap();
}

pub type GenericError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, GenericError>;

#[derive(Deserialize)]
pub struct SearchQueryParams {
    from: Option<u64>,
    limit: Option<u64>,
    exclude_count: Option<bool>,
    exclude_docs: Option<bool>,
    facet_field: Option<String>,
    facet_prefix: Option<Vec<String>>,
}

pub async fn route(req: Request<Body>, mut index_client: IndexClient) -> Result<Response<Body>> {
    match req.uri().path() {
        path => {
            if RE_DOCUMENTS.is_match(path) {
                let caps = RE_DOCUMENTS.captures(path).unwrap();
                let id: &str = &caps["id"];
                let id = String::from(id);
                match req.method() {
                    &Method::GET => match index_client.get(id) {
                        Ok(s) => Ok(Response::builder()
                            .status(StatusCode::OK)
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(s))
                            .unwrap()),
                        Err(e) => Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::from(e.to_string()))
                            .unwrap()),
                    },
                    &Method::PUT => {
                        let bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
                        let json_str = String::from_utf8(bytes.to_vec()).unwrap();
                        let mut value: Value = serde_json::from_str(json_str.as_str()).unwrap();
                        value["_id"] = Value::String(id);
                        let doc = serde_json::to_string(&value).unwrap();

                        match index_client.set(doc) {
                            Ok(_) => Ok(Response::builder()
                                .status(StatusCode::OK)
                                .body(Body::empty())
                                .unwrap()),
                            Err(e) => Ok(Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(Body::from(e.to_string()))
                                .unwrap()),
                        }
                    }
                    &Method::DELETE => match index_client.delete(id) {
                        Ok(_) => Ok(Response::builder()
                            .status(StatusCode::OK)
                            .body(Body::empty())
                            .unwrap()),
                        Err(e) => Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::from(e.to_string()))
                            .unwrap()),
                    },
                    _ => Ok(Response::builder()
                        .status(StatusCode::METHOD_NOT_ALLOWED)
                        .body(Body::empty())
                        .unwrap()),
                }
            } else if RE_DOCUMENTS_BULK.is_match(path) {
                match req.method() {
                    &Method::PUT => {
                        let bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
                        let json_str = String::from_utf8(bytes.to_vec()).unwrap();
                        let value: Value = serde_json::from_str(json_str.as_str()).unwrap();
                        let docs = serde_json::to_string(&value).unwrap();

                        match index_client.bulk_set(docs) {
                            Ok(_) => Ok(Response::builder()
                                .status(StatusCode::OK)
                                .body(Body::empty())
                                .unwrap()),
                            Err(e) => Ok(Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(Body::from(e.to_string()))
                                .unwrap()),
                        }
                    }
                    &Method::DELETE => {
                        let bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
                        let json_str = String::from_utf8(bytes.to_vec()).unwrap();
                        let value: Value = serde_json::from_str(json_str.as_str()).unwrap();
                        let docs = serde_json::to_string(&value).unwrap();

                        match index_client.bulk_delete(docs) {
                            Ok(_) => Ok(Response::builder()
                                .status(StatusCode::OK)
                                .body(Body::empty())
                                .unwrap()),
                            Err(e) => Ok(Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(Body::from(e.to_string()))
                                .unwrap()),
                        }
                    }
                    _ => Ok(Response::builder()
                        .status(StatusCode::METHOD_NOT_ALLOWED)
                        .body(Body::empty())
                        .unwrap()),
                }
            } else if RE_COMMIT.is_match(path) {
                match req.method() {
                    &Method::GET => match index_client.commit() {
                        Ok(_) => Ok(Response::builder()
                            .status(StatusCode::OK)
                            .body(Body::empty())
                            .unwrap()),
                        Err(e) => Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::from(e.to_string()))
                            .unwrap()),
                    },
                    _ => Ok(Response::builder()
                        .status(StatusCode::METHOD_NOT_ALLOWED)
                        .body(Body::empty())
                        .unwrap()),
                }
            } else if RE_ROLLBACK.is_match(path) {
                match req.method() {
                    &Method::GET => match index_client.rollback() {
                        Ok(_) => Ok(Response::builder()
                            .status(StatusCode::OK)
                            .body(Body::empty())
                            .unwrap()),
                        Err(e) => Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::from(e.to_string()))
                            .unwrap()),
                    },
                    _ => Ok(Response::builder()
                        .status(StatusCode::METHOD_NOT_ALLOWED)
                        .body(Body::empty())
                        .unwrap()),
                }
            } else if RE_MERGE.is_match(path) {
                match req.method() {
                    &Method::GET => match index_client.merge() {
                        Ok(_) => Ok(Response::builder()
                            .status(StatusCode::OK)
                            .body(Body::empty())
                            .unwrap()),
                        Err(e) => Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::from(e.to_string()))
                            .unwrap()),
                    },
                    _ => Ok(Response::builder()
                        .status(StatusCode::METHOD_NOT_ALLOWED)
                        .body(Body::empty())
                        .unwrap()),
                }
            } else if RE_SCHEMA.is_match(path) {
                match req.method() {
                    &Method::GET => match index_client.schema() {
                        Ok(s) => Ok(Response::builder()
                            .status(StatusCode::OK)
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(s))
                            .unwrap()),
                        Err(e) => Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::from(e.to_string()))
                            .unwrap()),
                    },
                    _ => Ok(Response::builder()
                        .status(StatusCode::METHOD_NOT_ALLOWED)
                        .body(Body::empty())
                        .unwrap()),
                }
            } else if RE_SEARCH.is_match(path) {
                match req.method() {
                    &Method::POST => {
                        let params =
                            serde_qs::from_str::<SearchQueryParams>(req.uri().query().unwrap())
                                .unwrap();
                        let mut from = 0;
                        if let Some(_from) = params.from {
                            from = _from;
                        }

                        let mut limit = 10;
                        if let Some(_limit) = params.limit {
                            limit = _limit;
                        }

                        let mut exclude_count = false;
                        if let Some(_exclude_count) = params.exclude_count {
                            exclude_count = _exclude_count;
                        }

                        let mut exclude_docs = false;
                        if let Some(_exclude_docs) = params.exclude_docs {
                            exclude_docs = _exclude_docs;
                        }

                        let mut facet_field = String::from("");
                        if let Some(_facet_field) = params.facet_field {
                            facet_field = _facet_field;
                        }

                        let mut facet_prefixes = Vec::new();
                        if let Some(_facet_prefix) = params.facet_prefix {
                            facet_prefixes = _facet_prefix;
                        }

                        let bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
                        let query = String::from_utf8(bytes.to_vec()).unwrap();

                        match index_client.search(
                            query.as_str(),
                            from,
                            limit,
                            exclude_count,
                            exclude_docs,
                            facet_field.as_str(),
                            facet_prefixes,
                        ) {
                            Ok(s) => Ok(Response::builder()
                                .status(StatusCode::OK)
                                .header(header::CONTENT_TYPE, "application/json")
                                .body(Body::from(s))
                                .unwrap()),
                            Err(e) => Ok(Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(Body::from(e.to_string()))
                                .unwrap()),
                        }
                    }
                    _ => Ok(Response::builder()
                        .status(StatusCode::METHOD_NOT_ALLOWED)
                        .body(Body::empty())
                        .unwrap()),
                }
            } else if RE_STATUS.is_match(path) {
                match req.method() {
                    &Method::GET => match index_client.status() {
                        Ok(s) => Ok(Response::builder()
                            .status(StatusCode::OK)
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(Body::from(s))
                            .unwrap()),
                        Err(e) => Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::from(e.to_string()))
                            .unwrap()),
                    },
                    _ => Ok(Response::builder()
                        .status(StatusCode::METHOD_NOT_ALLOWED)
                        .body(Body::empty())
                        .unwrap()),
                }
            } else {
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .unwrap())
            }
        }
    }
}
