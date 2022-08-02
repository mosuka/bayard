use axum::{
    body::Bytes,
    extract::{ContentLengthLimit, Path},
    response::IntoResponse,
    Extension, Json,
};
use http::StatusCode;
use serde_json::Value;
use tokio::io::AsyncBufReadExt;
use tonic::{transport::Channel, Request};
use tracing::error;

use crate::{
    index::metadata::Metadata,
    proto::index::{
        index_service_client::IndexServiceClient, query::Kind, sort::Order, CollectionKind,
        CommitRequest, CreateIndexRequest, DeleteDocumentsRequest, DeleteIndexRequest,
        GetIndexRequest, ModifyIndexRequest, PutDocumentsRequest, Query, RollbackRequest,
        SearchRequest, Sort,
    },
};

#[derive(Serialize, Deserialize, Debug)]
struct JsonQuery {
    kind: Kind,
    options: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonSort {
    field: String,
    order: Order,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonSearchRequest {
    query: JsonQuery,
    collection_kind: CollectionKind,
    sort: Option<JsonSort>,
    fields: Vec<String>,
    offset: i32,
    hits: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonDocument {
    id: String,
    score: Option<f32>,
    timestamp: Option<i64>,
    sort_value: Option<f64>,
    fields: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonSearchResponse {
    total_hits: i64,
    documents: Vec<JsonDocument>,
}

pub async fn create_index(
    Path(index): Path<String>,
    Json(request): Json<Metadata>,
    Extension(channel): Extension<Channel>,
) -> Result<impl IntoResponse, StatusCode> {
    let metadata_bytes = serde_json::to_vec(&request).map_err(|error| {
        error!(?error, "Failed to serialize schema.");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let req = CreateIndexRequest {
        name: index,
        metadata: metadata_bytes,
    };

    let mut client = IndexServiceClient::new(channel);
    let resp = client
        .create_index(Request::new(req))
        .await
        .map_err(|error| {
            error!(?error, "Failed to create index.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_inner();

    Ok((StatusCode::OK, Json(resp)))
}

pub async fn delete_index(
    Path(index): Path<String>,
    Extension(channel): Extension<Channel>,
) -> Result<impl IntoResponse, StatusCode> {
    let req = DeleteIndexRequest { name: index };

    let mut client = IndexServiceClient::new(channel);
    let resp = client
        .delete_index(Request::new(req))
        .await
        .map_err(|error| {
            error!(?error, "Failed to delete index.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_inner();

    Ok((StatusCode::OK, Json(resp)))
}

pub async fn get_index(
    Path(index): Path<String>,
    Extension(channel): Extension<Channel>,
) -> Result<impl IntoResponse, StatusCode> {
    let req = GetIndexRequest { name: index };

    let mut client = IndexServiceClient::new(channel);
    let resp = client
        .get_index(Request::new(req))
        .await
        .map_err(|error| {
            error!(?error, "Failed to get index metadata.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_inner();

    let metadata =
        serde_json::from_slice::<Metadata>(resp.metadata.as_slice()).map_err(|error| {
            error!(?error, "Failed to deserialize index metadata.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::OK, Json(metadata)))
}

pub async fn modify_index(
    Path(index): Path<String>,
    Json(request): Json<Metadata>,
    Extension(channel): Extension<Channel>,
) -> Result<impl IntoResponse, StatusCode> {
    let metadata_bytes = serde_json::to_vec(&request).map_err(|error| {
        error!(?error, "Failed to serialize index config.");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let req = ModifyIndexRequest {
        name: index,
        metadata: metadata_bytes,
    };

    let mut client = IndexServiceClient::new(channel);
    let resp = client
        .modify_index(Request::new(req))
        .await
        .map_err(|error| {
            error!(?error, "Failed to modify index metadata.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_inner();

    Ok((StatusCode::OK, Json(resp)))
}

pub async fn put_documents(
    Path(index): Path<String>,
    ContentLengthLimit(bytes): ContentLengthLimit<Bytes, 5_000_000>,
    Extension(channel): Extension<Channel>,
) -> Result<impl IntoResponse, StatusCode> {
    let docs_bytes = bytes.to_vec();

    let mut docs = Vec::new();
    let mut lines = docs_bytes.lines();
    while let Some(line) = lines.next_line().await.map_err(|error| {
        error!(?error, "Failed to read document.");
        StatusCode::INTERNAL_SERVER_ERROR
    })? {
        docs.push(line.into_bytes());
    }

    let req = PutDocumentsRequest {
        name: index,
        shard_id: "".to_string(),
        docs,
    };

    let mut client = IndexServiceClient::new(channel);
    let resp = client
        .put_documents(Request::new(req))
        .await
        .map_err(|error| {
            error!(?error, "Failed to put documents.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_inner();

    Ok((StatusCode::OK, Json(resp)))
}

pub async fn delete_documents(
    Path(index): Path<String>,
    ContentLengthLimit(bytes): ContentLengthLimit<Bytes, 5_000_000>,
    Extension(channel): Extension<Channel>,
) -> Result<impl IntoResponse, StatusCode> {
    let docs_bytes = bytes.to_vec();

    let mut doc_ids = Vec::new();
    let mut lines = docs_bytes.lines();
    while let Some(line) = lines.next_line().await.map_err(|error| {
        error!(?error, "Failed to read document.");
        StatusCode::INTERNAL_SERVER_ERROR
    })? {
        let doc = serde_json::from_str::<JsonDocument>(&line).map_err(|error| {
            error!(?error, "Failed to parse document.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        doc_ids.push(doc.id);
    }

    let req = DeleteDocumentsRequest {
        name: index,
        shard_id: "".to_string(),
        doc_ids,
    };

    let mut client = IndexServiceClient::new(channel);
    let resp = client
        .delete_documents(Request::new(req))
        .await
        .map_err(|error| {
            error!(?error, "Failed to delete documents");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_inner();

    Ok((StatusCode::OK, Json(resp)))
}

pub async fn commit(
    Path(index): Path<String>,
    Extension(channel): Extension<Channel>,
) -> Result<impl IntoResponse, StatusCode> {
    let req = CommitRequest {
        name: index,
        shard_id: "".to_string(),
    };

    let mut client = IndexServiceClient::new(channel);
    let resp = client
        .commit(Request::new(req))
        .await
        .map_err(|error| {
            error!(?error, "Failed to commit index.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_inner();

    Ok((StatusCode::OK, Json(resp)))
}

pub async fn rollback(
    Path(index): Path<String>,
    Extension(channel): Extension<Channel>,
) -> Result<impl IntoResponse, StatusCode> {
    let req = RollbackRequest {
        name: index,
        shard_id: "".to_string(),
    };

    let mut client = IndexServiceClient::new(channel);
    let resp = client
        .rollback(Request::new(req))
        .await
        .map_err(|error| {
            error!(?error, "Failed to rollback index.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_inner();

    Ok((StatusCode::OK, Json(resp)))
}

pub async fn search(
    Path(index): Path<String>,
    Json(request): Json<JsonSearchRequest>,
    Extension(channel): Extension<Channel>,
) -> Result<impl IntoResponse, StatusCode> {
    let options_bytes = serde_json::to_vec(&request.query.options).map_err(|error| {
        error!(?error, "Failed to deserialize query options.");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let query = Query {
        kind: request.query.kind as i32,
        options: options_bytes,
    };

    let sort = match request.sort {
        Some(sort) => Some(Sort {
            field: sort.field,
            order: sort.order as i32,
        }),
        None => None,
    };

    let req = SearchRequest {
        name: index,
        shard_id: "".to_string(),
        query: Some(query),
        collection_kind: request.collection_kind as i32,
        sort,
        fields: request.fields,
        offset: request.offset,
        hits: request.hits,
    };

    let mut client = IndexServiceClient::new(channel);

    let resp = client
        .search(Request::new(req))
        .await
        .map_err(|error| {
            error!(?error, "Failed to search index.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_inner();

    let mut docs = Vec::new();
    for tmp_doc in resp.documents {
        let fields =
            serde_json::from_slice::<Value>(tmp_doc.fields.as_slice()).map_err(|error| {
                error!(?error, "Failed to deserialize document.");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let doc = JsonDocument {
            id: tmp_doc.id,
            score: Some(tmp_doc.score),
            timestamp: Some(tmp_doc.timestamp),
            sort_value: Some(tmp_doc.sort_value),
            fields: Some(fields),
        };
        docs.push(doc);
    }

    let json = JsonSearchResponse {
        total_hits: resp.total_hits,
        documents: docs,
    };

    Ok((StatusCode::OK, Json(json)))
}
