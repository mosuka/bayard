use std::fmt;

pub mod cluster;
pub mod healthcheck;
pub mod index;

#[derive(Debug, Clone, Copy)]
pub enum ServiceErrorKind {
    IndexServiceCreationFailure,
    GetReplicasFailure,
}

impl ServiceErrorKind {
    pub fn with_error<E>(self, source: E) -> ServiceError
    where
        anyhow::Error: From<E>,
    {
        ServiceError {
            kind: self,
            source: From::from(source),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("ServerError(kind={kind:?}, source={source})")]
pub struct ServiceError {
    pub kind: ServiceErrorKind,
    #[source]
    source: anyhow::Error,
}

impl ServiceError {
    pub fn add_context<C>(self, ctx: C) -> Self
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        ServiceError {
            kind: self.kind,
            source: self.source.context(ctx),
        }
    }

    pub fn kind(&self) -> ServiceErrorKind {
        self.kind
    }
}
