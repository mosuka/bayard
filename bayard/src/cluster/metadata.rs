use std::{fmt, net::SocketAddr};

#[derive(Copy, Clone, Default, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Metadata {
    pub grpc_address: Option<SocketAddr>,
    pub http_address: Option<SocketAddr>,
}

impl fmt::Debug for Metadata {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Metadata")
            .field("grpc_address", &self.grpc_address)
            .field("http_address", &self.http_address)
            .finish()
    }
}
