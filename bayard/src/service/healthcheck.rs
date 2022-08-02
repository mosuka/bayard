use tonic::{Request, Response, Status};
use tracing::debug;

use crate::proto::healthcheck::{
    health_check_service_server::HealthCheckService as ProtoHealthCheckService,
    liveness_response::State as LivenessState, readiness_response::State as ReadinessState,
    LivenessRequest, LivenessResponse, ReadinessRequest, ReadinessResponse,
};

pub struct HealthCheckService {}

impl Default for HealthCheckService {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthCheckService {
    pub fn new() -> Self {
        Self {}
    }
}

#[tonic::async_trait]
impl ProtoHealthCheckService for HealthCheckService {
    async fn liveness(
        &self,
        request: Request<LivenessRequest>,
    ) -> Result<Response<LivenessResponse>, Status> {
        debug!(?request, "liveness");

        let reply = LivenessResponse {
            state: LivenessState::Alive as i32,
        };

        Ok(Response::new(reply))
    }

    async fn readiness(
        &self,
        request: Request<ReadinessRequest>,
    ) -> Result<Response<ReadinessResponse>, Status> {
        debug!(?request, "liveness");

        let reply = ReadinessResponse {
            state: ReadinessState::Ready as i32,
        };

        Ok(Response::new(reply))
    }
}
