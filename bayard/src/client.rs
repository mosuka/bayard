use std::{
    cmp::Ordering,
    collections::{hash_map::Entry, HashMap},
    fmt,
    net::SocketAddr,
    sync::Arc,
};

use exponential_backoff::Backoff;
use futures_util::future::try_join_all;
use http::Uri;
use tokio::{
    sync::RwLock,
    task::JoinHandle,
    time::{sleep, Duration},
};
use tokio_stream::{wrappers::WatchStream, StreamExt};
use tonic::{
    transport::{Channel, Endpoint},
    Code, Request, Response, Status,
};
use tracing::{error, info, warn};

use crate::{
    cluster::{member::Member, members::Members, membership::Membership},
    index::metastore::Metastore,
    proto::index::{
        index_service_client::IndexServiceClient, sort::Order, CommitRequest, CommitResponse,
        CreateIndexRequest, CreateIndexResponse, DeleteDocumentsRequest, DeleteDocumentsResponse,
        DeleteIndexRequest, DeleteIndexResponse, GetIndexRequest, GetIndexResponse,
        ModifyIndexRequest, ModifyIndexResponse, PutDocumentsRequest, PutDocumentsResponse,
        RollbackRequest, RollbackResponse, SearchRequest, SearchResponse,
    },
};

const EXPONENTIAL_BACKOFF_RETRIES: u32 = 5;
const EXPONENTIAL_BACKOFF_MIN_DURATION: Duration = Duration::from_millis(500);
const EXPONENTIAL_BACKOFF_MAX_DURATION: Duration = Duration::from_millis(3000);

#[derive(Debug, Clone, Copy)]
pub enum ClientErrorKind {
    UriCreationFailure,
    MetadataNotFound,
    GrpcAddressNotFound,
}

impl ClientErrorKind {
    pub fn with_error<E>(self, source: E) -> ClientError
    where
        anyhow::Error: From<E>,
    {
        ClientError {
            kind: self,
            source: From::from(source),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("ClientError(kind={kind:?}, source={source})")]
pub struct ClientError {
    pub kind: ClientErrorKind,
    #[source]
    source: anyhow::Error,
}

impl ClientError {
    pub fn add_context<C>(self, ctx: C) -> Self
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        ClientError {
            kind: self.kind,
            source: self.source.context(ctx),
        }
    }

    pub fn kind(&self) -> ClientErrorKind {
        self.kind
    }
}

pub fn create_index_client(
    grpc_address: &SocketAddr,
) -> Result<IndexServiceClient<Channel>, ClientError> {
    info!(?grpc_address, "Creating grpc client.");
    let uri = Uri::builder()
        .scheme("http")
        .authority(grpc_address.to_string())
        .path_and_query("/")
        .build()
        .map_err(|error| ClientErrorKind::UriCreationFailure.with_error(error))?;
    let channel = Endpoint::from(uri).connect_lazy();
    let client = IndexServiceClient::new(channel);

    Ok(client)
}

pub struct ClientPool {
    members: Arc<RwLock<Members>>,
    inner: Arc<RwLock<HashMap<SocketAddr, IndexServiceClient<Channel>>>>,
}

impl ClientPool {
    pub fn new(members: Vec<Member>, mut members_stream: WatchStream<Members>) -> Self {
        let mut inner = HashMap::new();
        for member in members.iter() {
            // Get metadata from member
            let metadata = if let Some(metadata) = member.metadata {
                metadata
            } else {
                error!(?member, "failed to get metadata");
                continue;
            };

            // Get gRPC address.
            let grpc_address = if let Some(grpc_address) = metadata.grpc_address {
                grpc_address
            } else {
                error!(?metadata, "failed to get gRPC address");
                continue;
            };

            // Create gRPC client.
            let grpc_client = create_index_client(&grpc_address).unwrap();
            inner.insert(member.addr, grpc_client);
        }
        let members = Members::init(members);

        let client_pool = Self {
            members: Arc::new(RwLock::new(members)),
            inner: Arc::new(RwLock::new(inner)),
        };

        let inner = Arc::clone(&client_pool.inner);
        let members = Arc::clone(&client_pool.members);
        tokio::spawn(async move {
            while let Some(received_members) = members_stream.next().await {
                let mut mut_inner = inner.write().await;
                let mut mut_members = members.write().await;

                // Add new gRPC clients.
                for member in received_members.iter() {
                    // Get metadata from member
                    let metadata = if let Some(metadata) = member.metadata {
                        metadata
                    } else {
                        error!(?member, "failed to get metadata");
                        continue;
                    };

                    if let Entry::Vacant(_entry) = mut_inner.entry(member.addr) {
                        // Get gRPC address.
                        let grpc_address = if let Some(grpc_address) = metadata.grpc_address {
                            grpc_address
                        } else {
                            error!(?metadata, "failed to get gRPC address");
                            continue;
                        };

                        // Create gRPC client.
                        let client = match create_index_client(&grpc_address) {
                            Ok(client) => client,
                            Err(err) => {
                                error!(?err, "failed to create gRPC client");
                                continue;
                            }
                        };

                        info!(?member, "add gRPC client");
                        mut_inner.insert(member.addr, client);
                    }

                    mut_members.push(member.clone());
                }

                // Remove dead gRPC clients.
                let cur = mut_members.clone();
                for member in cur.iter() {
                    if !received_members.contains(&member.addr) {
                        info!(?member, "delete gRPC client");
                        mut_inner.remove(&member.addr);
                        mut_members.remove(&member.addr);
                    }
                }

                info!(?mut_members, "update members");
            }
        });

        client_pool
    }

    pub async fn get(&self, addr: &SocketAddr) -> Option<IndexServiceClient<Channel>> {
        self.inner.read().await.get(addr).cloned()
    }

    pub async fn lookup(&self, key: &str) -> Option<IndexServiceClient<Channel>> {
        let addr = match self.members.read().await.lookup_member(key) {
            Some(member) => member.addr,
            None => return None,
        };
        self.get(&addr).await
    }

    pub async fn lookup_clients(&self, key: &str, num: usize) -> Vec<IndexServiceClient<Channel>> {
        let mut clients = Vec::new();
        for member in self.members.read().await.lookup_members(key, num) {
            if let Some(client) = self.get(&member.addr).await {
                clients.push(client);
            }
        }
        clients
    }

    pub async fn rotate(&self, key: &str, num: usize) -> Option<IndexServiceClient<Channel>> {
        let addr = match self.members.read().await.rotate_node(key, num) {
            Some(member) => member.addr,
            None => return None,
        };
        self.get(&addr).await
    }
}

pub struct Client {
    membership: Arc<Membership>,
    metastore: Arc<Metastore>,
    client_pool: Arc<ClientPool>,
}

impl Client {
    pub async fn new(membership: Arc<Membership>, metastore: Arc<Metastore>) -> Self {
        let members = membership.members().await.iter().cloned().collect();
        let members_stream = membership.watch_members();

        Self {
            membership,
            metastore,
            client_pool: Arc::new(ClientPool::new(members, members_stream)),
        }
    }

    pub async fn create_index(
        &self,
        request: Request<CreateIndexRequest>,
    ) -> Result<Response<CreateIndexResponse>, Status> {
        let local_member = self.membership.local_member().await;

        let mut client = match self.client_pool.get(&local_member.addr).await {
            Some(client) => client,
            None => {
                return Err(Status::new(
                    Code::Internal,
                    "failed to get local gRPC client",
                ))
            }
        };

        client.create_index(request).await
    }

    pub async fn delete_index(
        &self,
        request: Request<DeleteIndexRequest>,
    ) -> Result<Response<DeleteIndexResponse>, Status> {
        let local_member = self.membership.local_member().await;

        let mut client = match self.client_pool.get(&local_member.addr).await {
            Some(client) => client,
            None => {
                return Err(Status::new(
                    Code::Internal,
                    "failed to get local gRPC client",
                ))
            }
        };

        client.delete_index(request).await
    }

    pub async fn get_index(
        &self,
        request: Request<GetIndexRequest>,
    ) -> Result<Response<GetIndexResponse>, Status> {
        let local_member = self.membership.local_member().await;

        let mut client = match self.client_pool.get(&local_member.addr).await {
            Some(client) => client,
            None => {
                return Err(Status::new(
                    Code::Internal,
                    "failed to get local gRPC client",
                ))
            }
        };

        client.get_index(request).await
    }

    pub async fn modify_index(
        &self,
        request: Request<ModifyIndexRequest>,
    ) -> Result<Response<ModifyIndexResponse>, Status> {
        let local_member = self.membership.local_member().await;

        let mut client = match self.client_pool.get(&local_member.addr).await {
            Some(client) => client,
            None => {
                return Err(Status::new(
                    Code::Internal,
                    "failed to get local gRPC client",
                ))
            }
        };

        client.modify_index(request).await
    }

    pub async fn put_documents(
        &self,
        request: Request<PutDocumentsRequest>,
    ) -> Result<Response<PutDocumentsResponse>, Status> {
        let req = request.into_inner();

        let metadatas = self.metastore.metadatas().await;

        let index_name = req.name;

        let metadata = match metadatas.get(&index_name) {
            Some(meta) => meta,
            None => {
                return Err(Status::new(
                    Code::Internal,
                    format!("Failed to get metadata. index_name: {}", index_name),
                ))
            }
        };

        let num_replicas = match metadata.num_replicas() {
            Ok(replicas) => replicas,
            Err(_error) => {
                return Err(Status::new(
                    Code::Internal,
                    format!(
                        "Failed to get number of replicas. index_name: {}",
                        index_name
                    ),
                ))
            }
        };

        let shards = match metadata.shards() {
            Ok(shards) => shards,
            Err(_error) => {
                return Err(Status::new(
                    Code::Internal,
                    format!("Failed to get shards. index_name: {}", index_name),
                ))
            }
        };

        // Split documents into each shard.
        let mut docs_per_shard: HashMap<String, Vec<Vec<u8>>> = HashMap::new(); // shard_id -> documents
        for doc_bytes in req.docs {
            // Deserialize document to JSON value.
            let doc_value = match serde_json::from_slice::<serde_json::Value>(doc_bytes.as_slice())
            {
                Ok(doc_value) => doc_value,
                Err(error) => {
                    error!(?error, "Failed to deserialize document.");
                    continue;
                }
            };

            // Get document ID.
            let doc_id = match doc_value["id"].as_str() {
                Some(doc_id) => doc_id.to_string(),
                None => {
                    error!("Document does not have an id field.");
                    continue;
                }
            };

            // Get the shard ID to which the document ID is assigned.
            let shard_id = match shards.lookup_shard(&doc_id) {
                Some(shard) => shard.id.clone(),
                None => {
                    error!(?doc_id, "Failed to lookup shard for document");
                    continue;
                }
            };

            docs_per_shard
                .entry(shard_id)
                .or_insert_with(Vec::new)
                .push(doc_bytes);
        }

        // Send documents to nodes.
        let mut handles: Vec<JoinHandle<Result<tonic::Response<PutDocumentsResponse>, Status>>> =
            Vec::new();
        for (shard_id, shard_docs) in docs_per_shard {
            for mut client in self
                .client_pool
                .lookup_clients(&shard_id, num_replicas)
                .await
            {
                let distrib_req = PutDocumentsRequest {
                    name: index_name.clone(),
                    shard_id: shard_id.clone(),
                    docs: shard_docs.clone(),
                };

                let handle = tokio::spawn(async move {
                    let backoff = Backoff::new(
                        EXPONENTIAL_BACKOFF_RETRIES,
                        EXPONENTIAL_BACKOFF_MIN_DURATION,
                        EXPONENTIAL_BACKOFF_MAX_DURATION,
                    );
                    for duration in &backoff {
                        match client
                            .put_documents(Request::new(distrib_req.clone()))
                            .await
                        {
                            Ok(response) => {
                                return Ok(response);
                            }
                            Err(error) => {
                                error!(?error, "Failed to put documents.");
                            }
                        }
                        error!(?duration, "Retrying...");
                        sleep(duration).await;
                    }
                    error!(retries = ?EXPONENTIAL_BACKOFF_RETRIES, "Put documents retry count exceeded.");
                    Err(Status::new(
                        Code::Internal,
                        "Put documents retry count exceeded.",
                    ))
                });
                handles.push(handle);
            }
        }
        let responses = try_join_all(handles).await.map_err(|error| {
            Status::new(
                Code::Internal,
                format!("Failed to join all handles: {}", error),
            )
        })?;
        info!("Received put documents responses from nodes.");

        // Logging error.
        for response in responses {
            match response {
                Ok(_) => (),
                Err(error) => {
                    return Err(Status::new(
                        Code::Internal,
                        format!("Failed to put documents: {}", error),
                    ))
                }
            }
        }

        Ok(Response::new(PutDocumentsResponse {}))
    }

    pub async fn delete_documents(
        &self,
        request: Request<DeleteDocumentsRequest>,
    ) -> Result<Response<DeleteDocumentsResponse>, Status> {
        let req = request.into_inner();

        let metadatas = self.metastore.metadatas().await;

        let index_name = req.name;

        let metadata = match metadatas.get(&index_name) {
            Some(meta) => meta,
            None => {
                return Err(Status::new(
                    Code::Internal,
                    format!("Failed to get metadata. index_name: {}", index_name),
                ))
            }
        };

        let num_replicas = match metadata.num_replicas() {
            Ok(replicas) => replicas,
            Err(_error) => {
                return Err(Status::new(
                    Code::Internal,
                    format!(
                        "Failed to get number of replicas. index_name: {}",
                        index_name
                    ),
                ))
            }
        };

        let shards = match metadata.shards() {
            Ok(shards) => shards,
            Err(_error) => {
                return Err(Status::new(
                    Code::Internal,
                    format!("Failed to get shards. index_name: {}", index_name),
                ))
            }
        };

        let mut handles: Vec<JoinHandle<Result<tonic::Response<DeleteDocumentsResponse>, Status>>> =
            Vec::new();
        for shard in shards.iter() {
            for mut client in self
                .client_pool
                .lookup_clients(&shard.id, num_replicas)
                .await
            {
                let distrib_req = DeleteDocumentsRequest {
                    name: index_name.clone(),
                    shard_id: shard.id.clone(),
                    doc_ids: req.doc_ids.clone(),
                };

                let handle = tokio::spawn(async move {
                    let backoff = Backoff::new(
                        EXPONENTIAL_BACKOFF_RETRIES,
                        EXPONENTIAL_BACKOFF_MIN_DURATION,
                        EXPONENTIAL_BACKOFF_MAX_DURATION,
                    );
                    for duration in &backoff {
                        match client
                            .delete_documents(Request::new(distrib_req.clone()))
                            .await
                        {
                            Ok(response) => {
                                return Ok(response);
                            }
                            Err(error) => {
                                error!(?error, "Failed to delete documents.");
                            }
                        }
                        error!(?duration, "Retrying...");
                        sleep(duration).await;
                    }
                    error!(retries = ?EXPONENTIAL_BACKOFF_RETRIES, "Delete documents retry count exceeded.");
                    Err(Status::new(
                        Code::Internal,
                        "Delete documents retry count exceeded.",
                    ))
                });
                handles.push(handle);
            }
        }
        let responses = try_join_all(handles).await.map_err(|error| {
            Status::new(
                Code::Internal,
                format!("Failed to join all handles: {}", error),
            )
        })?;
        info!("Received delete documents responses from nodes.");

        // Logging error.
        for response in responses {
            match response {
                Ok(_) => (),
                Err(error) => {
                    return Err(Status::new(
                        Code::Internal,
                        format!("Failed to delete documents: {}", error),
                    ))
                }
            }
        }

        Ok(Response::new(DeleteDocumentsResponse {}))
    }

    pub async fn commit(
        &self,
        request: Request<CommitRequest>,
    ) -> Result<Response<CommitResponse>, Status> {
        let req = request.into_inner();

        let metadatas = self.metastore.metadatas().await;

        let index_name = req.name;

        let metadata = match metadatas.get(&index_name) {
            Some(meta) => meta,
            None => {
                return Err(Status::new(
                    Code::Internal,
                    format!("Failed to get metadata. index_name: {}", index_name),
                ))
            }
        };

        let num_replicas = match metadata.num_replicas() {
            Ok(replicas) => replicas,
            Err(_error) => {
                return Err(Status::new(
                    Code::Internal,
                    format!(
                        "Failed to get number of replicas. index_name: {}",
                        index_name
                    ),
                ))
            }
        };

        let shards = match metadata.shards() {
            Ok(shards) => shards,
            Err(_error) => {
                return Err(Status::new(
                    Code::Internal,
                    format!("Failed to get shards. index_name: {}", index_name),
                ))
            }
        };

        let mut handles: Vec<JoinHandle<Result<tonic::Response<CommitResponse>, Status>>> =
            Vec::new();
        for shard in shards.iter() {
            for mut client in self
                .client_pool
                .lookup_clients(&shard.id, num_replicas)
                .await
            {
                let distrib_req = CommitRequest {
                    name: index_name.clone(),
                    shard_id: shard.id.clone(),
                };

                let handle = tokio::spawn(async move {
                    let backoff = Backoff::new(
                        EXPONENTIAL_BACKOFF_RETRIES,
                        EXPONENTIAL_BACKOFF_MIN_DURATION,
                        EXPONENTIAL_BACKOFF_MAX_DURATION,
                    );
                    for duration in &backoff {
                        match client.commit(Request::new(distrib_req.clone())).await {
                            Ok(response) => {
                                return Ok(response);
                            }
                            Err(error) => {
                                error!(?error, "Failed to commit.");
                            }
                        }
                        error!(?duration, "Retrying...");
                        sleep(duration).await;
                    }
                    error!(retries = ?EXPONENTIAL_BACKOFF_RETRIES, "Commit retry count exceeded.");
                    Err(Status::new(Code::Internal, "Commit retry count exceeded."))
                });
                handles.push(handle);
            }
        }
        let responses = try_join_all(handles).await.map_err(|error| {
            Status::new(
                Code::Internal,
                format!("Failed to join all handles: {}", error),
            )
        })?;
        info!("Received commit responses from nodes.");

        // Logging error.
        for response in responses {
            match response {
                Ok(_) => (),
                Err(error) => {
                    return Err(Status::new(
                        Code::Internal,
                        format!("Failed to commit: {}", error),
                    ))
                }
            }
        }

        Ok(Response::new(CommitResponse {}))
    }

    pub async fn rollback(
        &self,
        request: Request<RollbackRequest>,
    ) -> Result<Response<RollbackResponse>, Status> {
        let req = request.into_inner();

        let metadatas = self.metastore.metadatas().await;

        let index_name = req.name;

        let metadata = match metadatas.get(&index_name) {
            Some(meta) => meta,
            None => {
                return Err(Status::new(
                    Code::Internal,
                    format!("Failed to get metadata. index_name: {}", index_name),
                ))
            }
        };

        let num_replicas = match metadata.num_replicas() {
            Ok(replicas) => replicas,
            Err(_error) => {
                return Err(Status::new(
                    Code::Internal,
                    format!(
                        "Failed to get number of replicas. index_name: {}",
                        index_name
                    ),
                ))
            }
        };

        let shards = match metadata.shards() {
            Ok(shards) => shards,
            Err(_error) => {
                return Err(Status::new(
                    Code::Internal,
                    format!("Failed to get shards. index_name: {}", index_name),
                ))
            }
        };

        let mut handles: Vec<JoinHandle<Result<tonic::Response<RollbackResponse>, Status>>> =
            Vec::new();
        for shard in shards.iter() {
            for mut client in self
                .client_pool
                .lookup_clients(&shard.id, num_replicas)
                .await
            {
                let distrib_req = RollbackRequest {
                    name: index_name.clone(),
                    shard_id: shard.id.clone(),
                };

                let handle = tokio::spawn(async move {
                    let backoff = Backoff::new(
                        EXPONENTIAL_BACKOFF_RETRIES,
                        EXPONENTIAL_BACKOFF_MIN_DURATION,
                        EXPONENTIAL_BACKOFF_MAX_DURATION,
                    );
                    for duration in &backoff {
                        match client.rollback(Request::new(distrib_req.clone())).await {
                            Ok(response) => {
                                return Ok(response);
                            }
                            Err(error) => {
                                error!(?error, "Failed to rollback.");
                            }
                        }
                        error!(?duration, "Retrying...");
                        sleep(duration).await;
                    }
                    error!(retries = ?EXPONENTIAL_BACKOFF_RETRIES, "Rollback retry count exceeded.");
                    Err(Status::new(
                        Code::Internal,
                        "Rollback retry count exceeded.",
                    ))
                });
                handles.push(handle);
            }
        }
        let responses = try_join_all(handles).await.map_err(|error| {
            Status::new(
                Code::Internal,
                format!("Failed to join all handles: {}", error),
            )
        })?;
        info!("Received rollback responses from nodes.");

        // Logging error.
        for response in responses {
            match response {
                Ok(_) => (),
                Err(error) => {
                    return Err(Status::new(
                        Code::Internal,
                        format!("Failed to commit: {}", error),
                    ))
                }
            }
        }

        Ok(Response::new(RollbackResponse {}))
    }

    pub async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let req = request.into_inner();

        let metadatas = self.metastore.metadatas().await;

        let index_name = req.name;

        let metadata = match metadatas.get(&index_name) {
            Some(meta) => meta,
            None => {
                return Err(Status::new(
                    Code::Internal,
                    format!("Failed to get metadata. index_name: {}", index_name),
                ))
            }
        };

        let num_replicas = match metadata.num_replicas() {
            Ok(replicas) => replicas,
            Err(_error) => {
                return Err(Status::new(
                    Code::Internal,
                    format!(
                        "Failed to get number of replicas. index_name: {}",
                        index_name
                    ),
                ))
            }
        };

        let shards = match metadata.shards() {
            Ok(shards) => shards,
            Err(_error) => {
                return Err(Status::new(
                    Code::Internal,
                    format!("Failed to get shards. index_name: {}", index_name),
                ))
            }
        };

        let mut handles: Vec<JoinHandle<Result<tonic::Response<SearchResponse>, Status>>> =
            Vec::new();
        for shard in shards.iter().cloned() {
            let distrib_req = SearchRequest {
                name: index_name.clone(),
                shard_id: shard.id.clone(),
                query: req.query.clone(),
                collection_kind: req.collection_kind,
                sort: req.sort.clone(),
                fields: req.fields.clone(),
                offset: 0,
                hits: req.hits + req.offset,
            };

            let client_pool = Arc::clone(&self.client_pool);

            let handle = tokio::spawn(async move {
                let backoff = Backoff::new(
                    EXPONENTIAL_BACKOFF_RETRIES,
                    Duration::from_micros(0),
                    Duration::from_micros(100),
                );
                for duration in &backoff {
                    match client_pool.rotate(&shard.id, num_replicas).await {
                        Some(mut client) => match client.search(distrib_req.clone()).await {
                            Ok(response) => {
                                return Ok(response);
                            }
                            Err(error) => {
                                error!(?error, "Failed to search.");
                            }
                        },
                        None => {
                            error!(shard_id = ?shard.id, "Failed to rotate client.");
                        }
                    }
                    error!(?duration, "Retrying...");
                    sleep(duration).await;
                }
                error!(retries = ?EXPONENTIAL_BACKOFF_RETRIES, "Search retry count exceeded.");
                Err(Status::new(Code::Internal, "Search retry count exceeded."))
            });
            handles.push(handle);
        }

        let responses = try_join_all(handles).await.map_err(|error| {
            Status::new(
                Code::Internal,
                format!("Failed to join all handles: {}", error),
            )
        })?;
        info!("Received search responses from nodes.");

        // Merge results.
        let mut total_hits = 0;
        let mut documents = Vec::new();

        for response in responses {
            match response {
                Ok(response) => {
                    let resp = response.into_inner();
                    total_hits += resp.total_hits;
                    documents.extend(resp.documents);
                }
                Err(error) => {
                    warn!(
                        ?index_name,
                        error = ? error,
                        "Failed to search index."
                    );
                }
            }
        }

        // Sort documents.
        if let Some(sort) = req.sort {
            match Order::from_i32(sort.order) {
                Some(order) => match order {
                    Order::Asc => {
                        documents.sort_by(|doc1, doc2| {
                            doc1.sort_value
                                .partial_cmp(&doc2.sort_value)
                                .unwrap_or(Ordering::Equal)
                        });
                    }
                    Order::Desc => {
                        documents.sort_by(|doc1, doc2| {
                            Ordering::reverse(
                                doc1.sort_value
                                    .partial_cmp(&doc2.sort_value)
                                    .unwrap_or(Ordering::Equal),
                            )
                        });
                    }
                    _ => {
                        return Err(Status::new(
                            Code::Internal,
                            format!("Unsupported sort order: {}", sort.order),
                        ))
                    }
                },
                None => {
                    return Err(Status::new(
                        Code::Internal,
                        format!("Failed to parse sort order: {}", sort.order),
                    ))
                }
            }
        } else {
            documents.sort_by(|doc1, doc2| {
                doc1.score
                    .partial_cmp(&doc2.score)
                    .unwrap_or(Ordering::Equal)
            });
        }

        // Calculate end-offset.
        let end_offset = if (documents.len() - req.offset as usize) < req.hits as usize {
            documents.len() as i32
        } else {
            req.offset + req.hits
        };

        Ok(Response::new(SearchResponse {
            total_hits,
            documents: documents[req.offset as usize..end_offset as usize].to_vec(),
        }))
    }
}
