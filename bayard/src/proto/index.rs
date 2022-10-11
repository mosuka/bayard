#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateIndexRequest {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="2")]
    pub metadata: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateIndexResponse {
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteIndexRequest {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteIndexResponse {
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetIndexRequest {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetIndexResponse {
    #[prost(bytes="vec", tag="1")]
    pub metadata: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ModifyIndexRequest {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="2")]
    pub metadata: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ModifyIndexResponse {
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PutDocumentsRequest {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub shard_id: ::prost::alloc::string::String,
    #[prost(bytes="vec", repeated, tag="3")]
    pub docs: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PutDocumentsResponse {
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteDocumentsRequest {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub shard_id: ::prost::alloc::string::String,
    #[prost(string, repeated, tag="3")]
    pub doc_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteDocumentsResponse {
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommitRequest {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub shard_id: ::prost::alloc::string::String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommitResponse {
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RollbackRequest {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub shard_id: ::prost::alloc::string::String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RollbackResponse {
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Query {
    #[prost(enumeration="query::Kind", tag="1")]
    pub kind: i32,
    #[prost(bytes="vec", tag="2")]
    pub options: ::prost::alloc::vec::Vec<u8>,
}
/// Nested message and enum types in `Query`.
pub mod query {
    #[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Kind {
        #[serde(rename = "unknown")]
        Unknown = 0,
        #[serde(rename = "all")]
        All = 1,
        #[serde(rename = "boolean")]
        Boolean = 2,
        #[serde(rename = "boost")]
        Boost = 3,
        #[serde(rename = "fuzzy_term")]
        FuzzyTerm = 4,
        #[serde(rename = "phrase")]
        Phrase = 5,
        #[serde(rename = "query_string")]
        QueryString = 6,
        #[serde(rename = "range")]
        Range = 7,
        #[serde(rename = "regex")]
        Regex = 8,
        #[serde(rename = "term")]
        Term = 9,
    }
    impl Kind {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Kind::Unknown => "UNKNOWN",
                Kind::All => "ALL",
                Kind::Boolean => "BOOLEAN",
                Kind::Boost => "BOOST",
                Kind::FuzzyTerm => "FUZZY_TERM",
                Kind::Phrase => "PHRASE",
                Kind::QueryString => "QUERY_STRING",
                Kind::Range => "RANGE",
                Kind::Regex => "REGEX",
                Kind::Term => "TERM",
            }
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Sort {
    #[prost(string, tag="1")]
    pub field: ::prost::alloc::string::String,
    #[prost(enumeration="sort::Order", tag="2")]
    pub order: i32,
}
/// Nested message and enum types in `Sort`.
pub mod sort {
    #[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Order {
        #[serde(rename = "unknown")]
        Unknown = 0,
        #[serde(rename = "asc")]
        Asc = 1,
        #[serde(rename = "desc")]
        Desc = 2,
    }
    impl Order {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Order::Unknown => "UNKNOWN",
                Order::Asc => "ASC",
                Order::Desc => "DESC",
            }
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Document {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(float, tag="2")]
    pub score: f32,
    #[prost(int64, tag="3")]
    pub timestamp: i64,
    #[prost(double, tag="4")]
    pub sort_value: f64,
    #[prost(bytes="vec", tag="5")]
    pub fields: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SearchRequest {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub shard_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag="3")]
    pub query: ::core::option::Option<Query>,
    #[prost(enumeration="CollectionKind", tag="4")]
    pub collection_kind: i32,
    #[prost(message, optional, tag="5")]
    pub sort: ::core::option::Option<Sort>,
    #[prost(string, repeated, tag="6")]
    pub fields: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(int32, tag="7")]
    pub offset: i32,
    #[prost(int32, tag="8")]
    pub hits: i32,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SearchResponse {
    #[prost(int64, tag="1")]
    pub total_hits: i64,
    #[prost(message, repeated, tag="2")]
    pub documents: ::prost::alloc::vec::Vec<Document>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CollectionKind {
    #[serde(rename = "unknown")]
    Unknown = 0,
    #[serde(rename = "count_and_top_docs")]
    CountAndTopDocs = 1,
    #[serde(rename = "count")]
    Count = 2,
    #[serde(rename = "top_docs")]
    TopDocs = 3,
}
impl CollectionKind {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            CollectionKind::Unknown => "UNKNOWN",
            CollectionKind::CountAndTopDocs => "COUNT_AND_TOP_DOCS",
            CollectionKind::Count => "COUNT",
            CollectionKind::TopDocs => "TOP_DOCS",
        }
    }
}
/// Generated client implementations.
pub mod index_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct IndexServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl IndexServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> IndexServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> IndexServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            IndexServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        pub async fn create_index(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateIndexRequest>,
        ) -> Result<tonic::Response<super::CreateIndexResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/index.IndexService/CreateIndex",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn delete_index(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteIndexRequest>,
        ) -> Result<tonic::Response<super::DeleteIndexResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/index.IndexService/DeleteIndex",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_index(
            &mut self,
            request: impl tonic::IntoRequest<super::GetIndexRequest>,
        ) -> Result<tonic::Response<super::GetIndexResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/index.IndexService/GetIndex",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn modify_index(
            &mut self,
            request: impl tonic::IntoRequest<super::ModifyIndexRequest>,
        ) -> Result<tonic::Response<super::ModifyIndexResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/index.IndexService/ModifyIndex",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn put_documents(
            &mut self,
            request: impl tonic::IntoRequest<super::PutDocumentsRequest>,
        ) -> Result<tonic::Response<super::PutDocumentsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/index.IndexService/PutDocuments",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn delete_documents(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteDocumentsRequest>,
        ) -> Result<tonic::Response<super::DeleteDocumentsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/index.IndexService/DeleteDocuments",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn commit(
            &mut self,
            request: impl tonic::IntoRequest<super::CommitRequest>,
        ) -> Result<tonic::Response<super::CommitResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/index.IndexService/Commit",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn rollback(
            &mut self,
            request: impl tonic::IntoRequest<super::RollbackRequest>,
        ) -> Result<tonic::Response<super::RollbackResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/index.IndexService/Rollback",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn search(
            &mut self,
            request: impl tonic::IntoRequest<super::SearchRequest>,
        ) -> Result<tonic::Response<super::SearchResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/index.IndexService/Search",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod index_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with IndexServiceServer.
    #[async_trait]
    pub trait IndexService: Send + Sync + 'static {
        async fn create_index(
            &self,
            request: tonic::Request<super::CreateIndexRequest>,
        ) -> Result<tonic::Response<super::CreateIndexResponse>, tonic::Status>;
        async fn delete_index(
            &self,
            request: tonic::Request<super::DeleteIndexRequest>,
        ) -> Result<tonic::Response<super::DeleteIndexResponse>, tonic::Status>;
        async fn get_index(
            &self,
            request: tonic::Request<super::GetIndexRequest>,
        ) -> Result<tonic::Response<super::GetIndexResponse>, tonic::Status>;
        async fn modify_index(
            &self,
            request: tonic::Request<super::ModifyIndexRequest>,
        ) -> Result<tonic::Response<super::ModifyIndexResponse>, tonic::Status>;
        async fn put_documents(
            &self,
            request: tonic::Request<super::PutDocumentsRequest>,
        ) -> Result<tonic::Response<super::PutDocumentsResponse>, tonic::Status>;
        async fn delete_documents(
            &self,
            request: tonic::Request<super::DeleteDocumentsRequest>,
        ) -> Result<tonic::Response<super::DeleteDocumentsResponse>, tonic::Status>;
        async fn commit(
            &self,
            request: tonic::Request<super::CommitRequest>,
        ) -> Result<tonic::Response<super::CommitResponse>, tonic::Status>;
        async fn rollback(
            &self,
            request: tonic::Request<super::RollbackRequest>,
        ) -> Result<tonic::Response<super::RollbackResponse>, tonic::Status>;
        async fn search(
            &self,
            request: tonic::Request<super::SearchRequest>,
        ) -> Result<tonic::Response<super::SearchResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct IndexServiceServer<T: IndexService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: IndexService> IndexServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for IndexServiceServer<T>
    where
        T: IndexService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/index.IndexService/CreateIndex" => {
                    #[allow(non_camel_case_types)]
                    struct CreateIndexSvc<T: IndexService>(pub Arc<T>);
                    impl<
                        T: IndexService,
                    > tonic::server::UnaryService<super::CreateIndexRequest>
                    for CreateIndexSvc<T> {
                        type Response = super::CreateIndexResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateIndexRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).create_index(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateIndexSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/index.IndexService/DeleteIndex" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteIndexSvc<T: IndexService>(pub Arc<T>);
                    impl<
                        T: IndexService,
                    > tonic::server::UnaryService<super::DeleteIndexRequest>
                    for DeleteIndexSvc<T> {
                        type Response = super::DeleteIndexResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteIndexRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).delete_index(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteIndexSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/index.IndexService/GetIndex" => {
                    #[allow(non_camel_case_types)]
                    struct GetIndexSvc<T: IndexService>(pub Arc<T>);
                    impl<
                        T: IndexService,
                    > tonic::server::UnaryService<super::GetIndexRequest>
                    for GetIndexSvc<T> {
                        type Response = super::GetIndexResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetIndexRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_index(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetIndexSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/index.IndexService/ModifyIndex" => {
                    #[allow(non_camel_case_types)]
                    struct ModifyIndexSvc<T: IndexService>(pub Arc<T>);
                    impl<
                        T: IndexService,
                    > tonic::server::UnaryService<super::ModifyIndexRequest>
                    for ModifyIndexSvc<T> {
                        type Response = super::ModifyIndexResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ModifyIndexRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).modify_index(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ModifyIndexSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/index.IndexService/PutDocuments" => {
                    #[allow(non_camel_case_types)]
                    struct PutDocumentsSvc<T: IndexService>(pub Arc<T>);
                    impl<
                        T: IndexService,
                    > tonic::server::UnaryService<super::PutDocumentsRequest>
                    for PutDocumentsSvc<T> {
                        type Response = super::PutDocumentsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::PutDocumentsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).put_documents(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PutDocumentsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/index.IndexService/DeleteDocuments" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteDocumentsSvc<T: IndexService>(pub Arc<T>);
                    impl<
                        T: IndexService,
                    > tonic::server::UnaryService<super::DeleteDocumentsRequest>
                    for DeleteDocumentsSvc<T> {
                        type Response = super::DeleteDocumentsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteDocumentsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).delete_documents(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteDocumentsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/index.IndexService/Commit" => {
                    #[allow(non_camel_case_types)]
                    struct CommitSvc<T: IndexService>(pub Arc<T>);
                    impl<
                        T: IndexService,
                    > tonic::server::UnaryService<super::CommitRequest>
                    for CommitSvc<T> {
                        type Response = super::CommitResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CommitRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).commit(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CommitSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/index.IndexService/Rollback" => {
                    #[allow(non_camel_case_types)]
                    struct RollbackSvc<T: IndexService>(pub Arc<T>);
                    impl<
                        T: IndexService,
                    > tonic::server::UnaryService<super::RollbackRequest>
                    for RollbackSvc<T> {
                        type Response = super::RollbackResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RollbackRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).rollback(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = RollbackSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/index.IndexService/Search" => {
                    #[allow(non_camel_case_types)]
                    struct SearchSvc<T: IndexService>(pub Arc<T>);
                    impl<
                        T: IndexService,
                    > tonic::server::UnaryService<super::SearchRequest>
                    for SearchSvc<T> {
                        type Response = super::SearchResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SearchRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).search(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SearchSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: IndexService> Clone for IndexServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: IndexService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: IndexService> tonic::server::NamedService for IndexServiceServer<T> {
        const NAME: &'static str = "index.IndexService";
    }
}
