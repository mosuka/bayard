use axum::{response::IntoResponse, Extension, Json};
use http::StatusCode;
use tonic::{transport::Channel, Request};
use tracing::error;

use crate::proto::cluster::{cluster_service_client::ClusterServiceClient, GetNodesRequest};

pub async fn nodes(
    Extension(channel): Extension<Channel>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut client = ClusterServiceClient::new(channel);

    let req = GetNodesRequest {};
    let resp = match client.get_nodes(Request::new(req)).await {
        Ok(resp) => resp.into_inner(),
        Err(error) => {
            error!("{}", error);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let json = match serde_json::to_value(resp) {
        Ok(json) => json,
        Err(error) => {
            error!("{}", error);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok((StatusCode::OK, Json(json)))
}
