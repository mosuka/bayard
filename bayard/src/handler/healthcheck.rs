use axum::{response::IntoResponse, Extension, Json};
use http::StatusCode;
use tonic::{transport::Channel, Request};
use tracing::error;

use crate::proto::healthcheck::{
    health_check_service_client::HealthCheckServiceClient,
    liveness_response::State as LivenessState, readiness_response::State as ReadinessState,
    LivenessRequest, ReadinessRequest,
};

pub async fn liveness(
    Extension(channel): Extension<Channel>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut client = HealthCheckServiceClient::new(channel);

    let req = LivenessRequest {};
    let resp = match client.liveness(Request::new(req)).await {
        Ok(resp) => resp.into_inner(),
        Err(error) => {
            error!("{}", error);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let state = match LivenessState::from_i32(resp.state) {
        Some(state) => state,
        None => {
            error!("Invalid liveness state: {}", resp.state);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let json = serde_json::json!({
        "state": state,
    });

    Ok((StatusCode::OK, Json(json)))
}

pub async fn readiness(
    Extension(channel): Extension<Channel>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut client = HealthCheckServiceClient::new(channel);

    let req = ReadinessRequest {};
    let resp = match client.readiness(Request::new(req)).await {
        Ok(resp) => resp.into_inner(),
        Err(error) => {
            error!("{}", error);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let state = match ReadinessState::from_i32(resp.state) {
        Some(state) => state,
        None => {
            error!("Invalid readiness state: {}", resp.state);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let json = serde_json::json!({
        "state": state,
    });

    Ok((StatusCode::OK, Json(json)))
}
