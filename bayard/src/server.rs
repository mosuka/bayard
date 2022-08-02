use std::{net::SocketAddr, sync::Arc};

use axum::{
    routing::{delete, get, post, put},
    Extension, Router, Server as AxumServer,
};
use futures_util::FutureExt;
use http::Uri;
use tokio::sync::{watch::Receiver, RwLock};
use tonic::transport::{Endpoint as TonicEndpoint, Server as TonicServer};
use tracing::info;

use crate::{
    client::Client,
    handler::{
        cluster::nodes,
        healthcheck::{liveness, readiness},
        index::{
            commit, create_index, delete_documents, delete_index, get_index, modify_index,
            put_documents, rollback, search,
        },
    },
    node::Node,
    proto::{
        cluster::cluster_service_server::ClusterServiceServer,
        healthcheck::health_check_service_server::HealthCheckServiceServer,
        index::index_service_server::IndexServiceServer,
    },
    service::{cluster::ClusterService, healthcheck::HealthCheckService, index::IndexService},
};

pub async fn handle_grpc_server(
    grpc_address: SocketAddr,
    node: Arc<RwLock<Node>>,
    client: Arc<Client>,
    mut rx_signal: Receiver<()>,
) {
    info!(?grpc_address, "Starting gRPC server.");
    TonicServer::builder()
        .add_service(ClusterServiceServer::new(ClusterService::new(Arc::clone(
            &node,
        ))))
        .add_service(HealthCheckServiceServer::new(HealthCheckService::new()))
        .add_service(IndexServiceServer::new(
            IndexService::new(Arc::clone(&node), Arc::clone(&client))
                .await
                .unwrap(),
        ))
        .serve_with_shutdown(
            grpc_address,
            rx_signal.changed().map(|s| {
                info!(?grpc_address, "Stopping gRPC server.");
                drop(s);
            }),
        )
        .await
        .unwrap();
}

pub async fn handle_http_server(
    http_address: SocketAddr,
    grpc_address: SocketAddr,
    mut rx_signal: Receiver<()>,
) {
    let uri = Uri::builder()
        .scheme("http")
        .authority(grpc_address.to_string())
        .path_and_query("/")
        .build()
        .unwrap();
    let channel = TonicEndpoint::from(uri).connect_lazy();

    // Create the router.
    let app = Router::new()
        .route("/healthcheck/livez", get(liveness))
        .route("/healthcheck/readyz", get(readiness))
        .route("/cluster/nodes", get(nodes))
        .route("/indices/:index", put(create_index))
        .route("/indices/:index", delete(delete_index))
        .route("/indices/:index", get(get_index))
        .route("/indices/:index", post(modify_index))
        .route("/indices/:index/documents", put(put_documents))
        .route("/indices/:index/documents", delete(delete_documents))
        .route("/indices/:index/commit", get(commit))
        .route("/indices/:index/rollback", get(rollback))
        .route("/indices/:index/search", post(search))
        .layer(Extension(channel));

    info!(?http_address, "Starting HTTP server.");
    AxumServer::bind(&http_address)
        .serve(app.into_make_service())
        .with_graceful_shutdown(rx_signal.changed().map(|s| {
            info!(?http_address, "Stopping HTTP server.");
            drop(s);
        }))
        .await
        .unwrap();
}
