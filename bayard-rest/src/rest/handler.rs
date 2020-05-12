use actix_web::{delete, get, post, put, web, Error, HttpRequest, HttpResponse};
use serde::Deserialize;
use serde_json::Value;

use crate::rest::server::AppState;

#[get("/v1/documents/{id}")]
pub async fn get(state: AppState, id: web::Path<String>) -> Result<HttpResponse, Error> {
    match state.index_client.lock().unwrap().get(id.into_inner()) {
        Ok(s) => {
            let res = HttpResponse::Ok().body(s);
            Ok(res)
        }
        Err(e) => {
            let res = HttpResponse::InternalServerError().body(format!("{}", e));
            Ok(res)
        }
    }
}

#[put("/v1/documents/{id}")]
pub async fn set(
    state: AppState,
    body: web::Bytes,
    id: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let json_str = String::from_utf8(body.to_vec()).unwrap();
    let mut value: Value = serde_json::from_str(json_str.as_str()).unwrap();
    value["_id"] = Value::String(id.into_inner());

    let doc = serde_json::to_string(&value).unwrap();

    match state.index_client.lock().unwrap().set(doc) {
        Ok(_) => {
            let res = HttpResponse::Ok().await.unwrap();
            Ok(res)
        }
        Err(e) => {
            let res = HttpResponse::InternalServerError().body(format!("{}", e));
            Ok(res)
        }
    }
}

#[delete("/v1/documents/{id}")]
pub async fn delete(state: AppState, id: web::Path<String>) -> Result<HttpResponse, Error> {
    match state.index_client.lock().unwrap().delete(id.into_inner()) {
        Ok(_) => {
            let res = HttpResponse::Ok().await.unwrap();
            Ok(res)
        }
        Err(e) => {
            let res = HttpResponse::InternalServerError().body(format!("{}", e));
            Ok(res)
        }
    }
}

#[put("/v1/documents")]
pub async fn bulk_set(state: AppState, body: web::Bytes) -> Result<HttpResponse, Error> {
    let docs = String::from_utf8(body.to_vec()).unwrap();

    match state.index_client.lock().unwrap().bulk_set(docs) {
        Ok(_) => {
            let res = HttpResponse::Ok().await.unwrap();
            Ok(res)
        }
        Err(e) => {
            let res = HttpResponse::InternalServerError().body(format!("{}", e));
            Ok(res)
        }
    }
}

#[delete("/v1/documents")]
pub async fn bulk_delete(state: AppState, body: web::Bytes) -> Result<HttpResponse, Error> {
    let docs = String::from_utf8(body.to_vec()).unwrap();

    match state.index_client.lock().unwrap().bulk_delete(docs) {
        Ok(_) => {
            let res = HttpResponse::Ok().await.unwrap();
            Ok(res)
        }
        Err(e) => {
            let res = HttpResponse::InternalServerError().body(format!("{}", e));
            Ok(res)
        }
    }
}

#[get("/v1/commit")]
pub async fn commit(state: AppState) -> Result<HttpResponse, Error> {
    match state.index_client.lock().unwrap().commit() {
        Ok(_) => {
            let res = HttpResponse::Ok().await.unwrap();
            Ok(res)
        }
        Err(e) => {
            let res = HttpResponse::InternalServerError().body(format!("{}", e));
            Ok(res)
        }
    }
}

#[get("/v1/rollback")]
pub async fn rollback(state: AppState) -> Result<HttpResponse, Error> {
    match state.index_client.lock().unwrap().rollback() {
        Ok(_) => {
            let res = HttpResponse::Ok().await.unwrap();
            Ok(res)
        }
        Err(e) => {
            let res = HttpResponse::InternalServerError().body(format!("{}", e));
            Ok(res)
        }
    }
}

#[get("/v1/merge")]
pub async fn merge(state: AppState) -> Result<HttpResponse, Error> {
    match state.index_client.lock().unwrap().merge() {
        Ok(_) => {
            let res = HttpResponse::Ok().await.unwrap();
            Ok(res)
        }
        Err(e) => {
            let res = HttpResponse::InternalServerError().body(format!("{}", e));
            Ok(res)
        }
    }
}

#[get("/v1/schema")]
pub async fn schema(state: AppState) -> Result<HttpResponse, Error> {
    match state.index_client.lock().unwrap().schema() {
        Ok(_) => {
            let res = HttpResponse::Ok().await.unwrap();
            Ok(res)
        }
        Err(e) => {
            let res = HttpResponse::InternalServerError().body(format!("{}", e));
            Ok(res)
        }
    }
}

#[derive(Deserialize)]
pub struct SearchQueryParams {
    from: Option<u64>,
    limit: Option<u64>,
    exclude_count: Option<bool>,
    exclude_docs: Option<bool>,
    facet_field: Option<String>,
    facet_prefix: Option<Vec<String>>,
}

#[post("/v1/search")]
pub async fn search(
    state: AppState,
    req: HttpRequest,
    body: web::Bytes,
) -> Result<HttpResponse, Error> {
    let params = serde_qs::from_str::<SearchQueryParams>(&req.query_string()).unwrap();

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

    let query = String::from_utf8(body.to_vec()).unwrap();

    match state.index_client.lock().unwrap().search(
        query.as_str(),
        from,
        limit,
        exclude_count,
        exclude_docs,
        facet_field.as_str(),
        facet_prefixes,
    ) {
        Ok(s) => {
            let res = HttpResponse::Ok().body(s);
            Ok(res)
        }
        Err(e) => {
            let res = HttpResponse::InternalServerError().body(format!("{}", e));
            Ok(res)
        }
    }
}

#[get("/v1/status")]
pub async fn status(state: AppState) -> Result<HttpResponse, Error> {
    match state.index_client.lock().unwrap().status() {
        Ok(s) => {
            let res = HttpResponse::Ok().body(s);
            Ok(res)
        }
        Err(e) => {
            let res = HttpResponse::InternalServerError().body(format!("{}", e));
            Ok(res)
        }
    }
}
