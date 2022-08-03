use std::sync::Arc;

use tokio::sync::RwLock;
use tonic::{Request, Response, Status};
use tracing::debug;

use crate::{
    node::Node,
    proto::cluster::{
        cluster_service_server::ClusterService as ProtoClusterService, GetNodesRequest,
        GetNodesResponse, Metadata, Node as ProtoNode,
    },
};

pub struct ClusterService {
    node: Arc<RwLock<Node>>,
}

impl ClusterService {
    pub fn new(node: Arc<RwLock<Node>>) -> Self {
        Self { node }
    }
}

#[tonic::async_trait]
impl ProtoClusterService for ClusterService {
    async fn get_nodes(
        &self,
        request: Request<GetNodesRequest>,
    ) -> Result<Response<GetNodesResponse>, Status> {
        debug!(?request, "get_nodes");

        let mut nodes = Vec::new();

        // Add local node.
        let local_member = self.node.read().await.local_member().await;

        if let Some(metadata) = local_member.metadata {
            nodes.push(ProtoNode {
                address: local_member.addr.to_string(),
                metadata: Some(Metadata {
                    grpc_address: metadata
                        .grpc_address
                        .map_or(String::new(), |addr| addr.to_string()),
                    http_address: metadata
                        .http_address
                        .map_or(String::new(), |addr| addr.to_string()),
                }),
            });
        } else {
            nodes.push(ProtoNode {
                address: local_member.addr.to_string(),
                metadata: None,
            });
        }

        // Add remote nodes.
        let remote_members = self.node.read().await.remote_members().await;
        for remote_member in remote_members {
            if let Some(metadata) = remote_member.metadata {
                nodes.push(ProtoNode {
                    address: remote_member.addr.to_string(),
                    metadata: Some(Metadata {
                        grpc_address: metadata
                            .grpc_address
                            .map_or(String::new(), |addr| addr.to_string()),
                        http_address: metadata
                            .http_address
                            .map_or(String::new(), |addr| addr.to_string()),
                    }),
                });
            } else {
                nodes.push(ProtoNode {
                    address: remote_member.addr.to_string(),
                    metadata: None,
                });
            }
        }

        let reply = GetNodesResponse { nodes };

        Ok(Response::new(reply))
    }
}
