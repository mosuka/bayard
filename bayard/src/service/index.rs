use std::sync::Arc;

use tokio::{sync::RwLock, time::Instant};
use tonic::{Code, Request, Response, Status};
use tracing::info;

use crate::{
    client::Client,
    node::Node,
    proto::index::{
        index_service_server::IndexService as ProtoIndexService, CommitRequest, CommitResponse,
        CreateIndexRequest, CreateIndexResponse, DecrementShardsRequest, DecrementShardsResponse,
        DeleteDocumentsRequest, DeleteDocumentsResponse, DeleteIndexRequest, DeleteIndexResponse,
        GetIndexRequest, GetIndexResponse, IncrementShardsRequest, IncrementShardsResponse,
        ModifyIndexRequest, ModifyIndexResponse, PutDocumentsRequest, PutDocumentsResponse,
        RollbackRequest, RollbackResponse, SearchRequest, SearchResponse,
    },
};

use super::ServiceError;

pub struct IndexService {
    node: Arc<RwLock<Node>>,
    client: Arc<Client>,
}

impl IndexService {
    pub async fn new(node: Arc<RwLock<Node>>, client: Arc<Client>) -> Result<Self, ServiceError> {
        Ok(Self { node, client })
    }
}

#[tonic::async_trait]
impl ProtoIndexService for IndexService {
    async fn create_index(
        &self,
        request: Request<CreateIndexRequest>,
    ) -> Result<tonic::Response<CreateIndexResponse>, Status> {
        let now = Instant::now();

        let req = request.into_inner();

        let resp = match self.node.read().await.create_index(req).await {
            Ok(resp) => Ok(Response::new(resp)),
            Err(err) => Err(Status::new(Code::Internal, err.to_string())),
        };

        info!(elapsed = ?now.elapsed(), "Create index completed.");

        resp
    }

    async fn delete_index(
        &self,
        request: Request<DeleteIndexRequest>,
    ) -> Result<tonic::Response<DeleteIndexResponse>, Status> {
        let now = Instant::now();

        let req = request.into_inner();

        let resp = match self.node.read().await.delete_index(req).await {
            Ok(resp) => Ok(Response::new(resp)),
            Err(err) => Err(Status::new(Code::Internal, err.to_string())),
        };

        info!(elapsed = ?now.elapsed(), "Delete index completed.");

        resp
    }

    async fn get_index(
        &self,
        request: Request<GetIndexRequest>,
    ) -> Result<tonic::Response<GetIndexResponse>, Status> {
        let now = Instant::now();

        let req = request.into_inner();

        let resp = match self.node.read().await.get_index(req).await {
            Ok(resp) => Ok(Response::new(resp)),
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        };

        info!(elapsed = ?now.elapsed(), "Modify index completed.");

        resp
    }

    async fn modify_index(
        &self,
        request: Request<ModifyIndexRequest>,
    ) -> Result<tonic::Response<ModifyIndexResponse>, Status> {
        let now = Instant::now();

        let req = request.into_inner();

        let resp = match self.node.read().await.modify_index(req).await {
            Ok(resp) => Ok(Response::new(resp)),
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        };

        info!(elapsed = ?now.elapsed(), "Modify index completed.");

        resp
    }

    async fn increment_shards(
        &self,
        request: Request<IncrementShardsRequest>,
    ) -> Result<tonic::Response<IncrementShardsResponse>, Status> {
        let now = Instant::now();

        let req = request.into_inner();

        let resp = match self.node.read().await.increment_num_shards(req).await {
            Ok(resp) => Ok(Response::new(resp)),
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        };

        info!(elapsed = ?now.elapsed(), "Incrementing shards completed.");

        resp
    }

    async fn decrement_shards(
        &self,
        request: Request<DecrementShardsRequest>,
    ) -> Result<tonic::Response<DecrementShardsResponse>, Status> {
        let now = Instant::now();

        let req = request.into_inner();

        let resp = match self.node.read().await.decrement_num_shards(req).await {
            Ok(resp) => Ok(Response::new(resp)),
            Err(e) => Err(Status::new(Code::Internal, e.to_string())),
        };

        info!(elapsed = ?now.elapsed(), "decrementing shards completed.");

        resp
    }

    async fn put_documents(
        &self,
        request: Request<PutDocumentsRequest>,
    ) -> Result<tonic::Response<PutDocumentsResponse>, Status> {
        let now = Instant::now();

        let req = request.into_inner();

        let resp = if req.shard_id.is_empty() {
            self.client.put_documents(Request::new(req)).await
        } else {
            let resp = self
                .node
                .read()
                .await
                .put_documents(req)
                .await
                .map_err(|error| {
                    Status::new(
                        Code::Internal,
                        format!("Failed to put documents: error = {:?}", error),
                    )
                })?;
            Ok(Response::new(resp))
        };

        info!(elapsed = ?now.elapsed(), "Put documents completed.");

        resp
    }

    async fn delete_documents(
        &self,
        request: Request<DeleteDocumentsRequest>,
    ) -> Result<tonic::Response<DeleteDocumentsResponse>, Status> {
        let now = Instant::now();

        let req = request.into_inner();

        let resp = if req.shard_id.is_empty() {
            self.client.delete_documents(Request::new(req)).await
        } else {
            let resp = self
                .node
                .read()
                .await
                .delete_documents(req)
                .await
                .map_err(|error| {
                    Status::new(
                        Code::Internal,
                        format!("Failed to delete documents: error = {:?}", error),
                    )
                })?;
            Ok(Response::new(resp))
        };

        info!(elapsed = ?now.elapsed(), "Delete documents completed.");

        resp
    }

    async fn commit(
        &self,
        request: Request<CommitRequest>,
    ) -> Result<tonic::Response<CommitResponse>, Status> {
        let now = Instant::now();

        let req = request.into_inner();

        let resp = if req.shard_id.is_empty() {
            self.client.commit(Request::new(req)).await
        } else {
            let resp = self.node.read().await.commit(req).await.map_err(|error| {
                Status::new(
                    Code::Internal,
                    format!("Failed to commit: error = {:?}", error),
                )
            })?;
            Ok(Response::new(resp))
        };

        info!(elapsed = ?now.elapsed(), "Commit completed.");

        resp
    }

    async fn rollback(
        &self,
        request: Request<RollbackRequest>,
    ) -> Result<tonic::Response<RollbackResponse>, Status> {
        let now = Instant::now();

        let req = request.into_inner();

        let resp = if req.shard_id.is_empty() {
            self.client.rollback(Request::new(req)).await
        } else {
            let resp = self
                .node
                .read()
                .await
                .rollback(req)
                .await
                .map_err(|error| {
                    Status::new(
                        Code::Internal,
                        format!("Failed to commit: error = {:?}", error),
                    )
                })?;
            Ok(Response::new(resp))
        };

        info!(elapsed = ?now.elapsed(), "Rollback completed.");

        resp
    }

    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<tonic::Response<SearchResponse>, Status> {
        let now = Instant::now();

        let req = request.into_inner();

        let resp = if req.shard_id.is_empty() {
            self.client.search(Request::new(req)).await
        } else {
            let search_resp = self.node.read().await.search(req).await.map_err(|error| {
                Status::new(
                    Code::Internal,
                    format!("Failed to commit: error = {:?}", error),
                )
            })?;
            Ok(Response::new(search_resp))
        };

        info!(elapsed = ?now.elapsed(), "Search completed.");

        resp
    }
}
