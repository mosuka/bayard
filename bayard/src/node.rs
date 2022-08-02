pub mod index;
pub mod search;

use std::{fmt, path::PathBuf, sync::Arc};

use dashmap::DashMap;
use tantivy::{schema::FieldType, DateTime, Index, IndexReader, IndexWriter, Term};
use time::OffsetDateTime;
use tokio::fs;
use tokio_stream::{wrappers::WatchStream, StreamExt};
use tracing::{debug, error, info, warn};

use crate::{
    cluster::{
        member::Member,
        message::{
            CreateIndexMessage, DeleteIndexMessage, Message, MessageKind, ModifyIndexMessage,
        },
    },
    cluster::{members::Members, membership::Membership},
    common::{read_file, remove_file},
    index::{
        metadata::{save_index_metadata, Metadata as IndexMetadata},
        metastore::Metastore,
        shard::Shard,
        shards::Shards,
        DOC_ID_FIELD_NAME, DOC_TIMESTAMP_FIELD_NAME, INDEX_METADATA_FILE, SHARDS_DIR,
    },
    node::index::delete_index,
    proto::index::{
        sort::Order, CollectionKind, CommitRequest, CommitResponse, CreateIndexRequest,
        CreateIndexResponse, DeleteDocumentsRequest, DeleteDocumentsResponse, DeleteIndexRequest,
        DeleteIndexResponse, GetIndexRequest, GetIndexResponse, ModifyIndexRequest,
        ModifyIndexResponse, PutDocumentsRequest, PutDocumentsResponse, RollbackRequest,
        RollbackResponse, SearchRequest, SearchResponse,
    },
    search::query::create_query,
};

use self::{
    index::{create_index, index_exists, open_index},
    search::{
        search_order_by_date_asc, search_order_by_date_desc, search_order_by_f64_asc,
        search_order_by_f64_desc, search_order_by_i64_asc, search_order_by_i64_desc,
        search_order_by_score_desc, search_order_by_u64_asc, search_order_by_u64_desc,
    },
};

#[derive(Debug, Clone, Copy)]
pub enum NodeErrorKind {
    MetadataError,
    InvalidPath,
    IndexNotFound,
    IndexExistenceFailure,
    IndexExists,
    ShardNotFound,
    DocumentDeserializationFailure,
    DocumentSerializationFailure,
    DocumentParseFailure,
    MessageSerializationFailure,
    MembersSerializationFailure,
    FieldNotFound,
    IndexCreationFailure,
    IndexOpenFailure,
    IndexDeletionFailure,
    IndexCommitFailure,
    IndexRollbackFailure,
    IndexSearchFailure,
    FileWriteFailure,
    FileRemoveFailure,
    FileReadFailure,
    DirectoryCreationFailure,
    DirectoryReadFailure,
    SearchFailre,
    SocketBindingFailure,
    ModifyIndexFailure,
    IndexMetadataSerializationFailure,
    SchemaDeserializationFailure,
    SchemaSerializationFailure,
    IndexDirectoryCreationFailure,
    AnalyzerRegistrationFailure,
    IndexConfigDoesNotExist,
}

impl NodeErrorKind {
    pub fn with_error<E>(self, source: E) -> NodeError
    where
        anyhow::Error: From<E>,
    {
        NodeError {
            kind: self,
            source: From::from(source),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("NodeError(kind={kind:?}, source={source})")]
pub struct NodeError {
    pub kind: NodeErrorKind,
    #[source]
    source: anyhow::Error,
}

impl NodeError {
    pub fn add_context<C>(self, ctx: C) -> Self
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        NodeError {
            kind: self.kind,
            source: self.source.context(ctx),
        }
    }

    pub fn kind(&self) -> NodeErrorKind {
        self.kind
    }
}

pub struct Node {
    membership: Arc<Membership>,
    metastore: Arc<Metastore>,
    indices_dir: PathBuf,
    indices: Arc<DashMap<String, DashMap<String, Index>>>, // index_name -> shard_id -> index
    index_writers: Arc<DashMap<String, DashMap<String, IndexWriter>>>, // index_name -> shard_id -> index_writer
    index_readers: Arc<DashMap<String, DashMap<String, IndexReader>>>, // index_name -> shard_id -> index_reader
}

impl Node {
    pub async fn new(
        membership: Arc<Membership>,
        metastore: Arc<Metastore>,
        indices_dir: PathBuf,
    ) -> Result<Self, NodeError> {
        if !indices_dir.exists() {
            info!(?indices_dir, "Creating indices directory.");
            fs::create_dir_all(&indices_dir)
                .await
                .map_err(|error| NodeErrorKind::DirectoryCreationFailure.with_error(error))?;
        }

        let indices: DashMap<String, DashMap<String, Index>> = DashMap::new();
        let index_writers: DashMap<String, DashMap<String, IndexWriter>> = DashMap::new();
        let index_readers: DashMap<String, DashMap<String, IndexReader>> = DashMap::new();

        let node = Self {
            membership,
            metastore,
            indices_dir,
            indices: Arc::new(indices),
            index_writers: Arc::new(index_writers),
            index_readers: Arc::new(index_readers),
        };

        node.handle_metadatas().await;
        node.handle_messages().await;

        Ok(node)
    }

    async fn handle_metadatas(&self) {
        let mut stream = self.metastore.watch();
        let membership = Arc::clone(&self.membership);
        let metastore = Arc::clone(&self.metastore);
        let indices = Arc::clone(&self.indices);
        let index_writers = Arc::clone(&self.index_writers);
        let index_readers = Arc::clone(&self.index_readers);
        let indices_dir = Arc::new(self.indices_dir.clone());

        tokio::spawn(async move {
            let local_addr = membership.local_member().await.addr;

            while let Some(metadatas) = stream.next().await {
                info!("Received metadatas.");

                for (index_name, metadata) in metadatas.iter() {
                    let shards = match metadata.shards() {
                        Ok(shards) => shards,
                        Err(error) => {
                            error!(?index_name, ?error, "Failed to get shards.");
                            continue;
                        }
                    };

                    let num_replicas = match metadata.num_replicas() {
                        Ok(num_replicas) => num_replicas,
                        Err(error) => {
                            error!(?index_name, ?error, "Failed to get number of replicas.");
                            continue;
                        }
                    };

                    for shard in shards.iter() {
                        let is_assigned_shard = membership
                            .members()
                            .await
                            .lookup_members(&shard.id, num_replicas)
                            .map(|members| members.addr)
                            .any(|addr| addr == local_addr);
                        if is_assigned_shard {
                            info!(?index_name, shard_id = ?shard.id, "Shard is assigned to this node.");

                            // Check index object existence.
                            let index = if indices
                                .entry(index_name.clone())
                                .or_insert_with(DashMap::new)
                                .contains_key(&shard.id)
                            {
                                debug!(?index_name, shard_id = ?shard.id, "Index object already exists.");
                                match indices
                                    .entry(index_name.clone())
                                    .or_insert_with(DashMap::new)
                                    .get(&shard.id)
                                {
                                    Some(index) => index.clone(),
                                    None => {
                                        error!(?index_name, shard_id = ?shard.id, "Index object not found.");
                                        continue;
                                    }
                                }
                            } else {
                                // Make shard directory.
                                let shard_dir = indices_dir
                                    .join(index_name)
                                    .join(SHARDS_DIR)
                                    .join(&shard.id);

                                // Check shard index existence.
                                let is_shard_index_exist = match index_exists(&shard_dir).await {
                                    Ok(is_shard_index_exist) => is_shard_index_exist,
                                    Err(error) => {
                                        error!(?index_name, ?shard.id, ?error, "Failed to check shard index existence.");
                                        continue;
                                    }
                                };

                                // Create index object.
                                let index = if is_shard_index_exist {
                                    // Get analyzers.
                                    let analyzers = match metadata.analyzers() {
                                        Ok(analyzers) => analyzers,
                                        Err(error) => {
                                            error!(?index_name, ?shard.id, ?error, "Failed to get analyzers.");
                                            continue;
                                        }
                                    };

                                    // Open index.
                                    info!(?shard_dir, "Opening shard index.");
                                    match open_index(&shard_dir, &analyzers).await {
                                        Ok(index) => index,
                                        Err(error) => {
                                            error!(
                                                ?shard_dir,
                                                ?error,
                                                "Failed to open shard index."
                                            );
                                            continue;
                                        }
                                    }
                                } else {
                                    // Get schema.
                                    let schema = match metadata.schema() {
                                        Ok(schema) => schema,
                                        Err(error) => {
                                            error!(?index_name, ?shard.id, ?error, "Failed to get schema.");
                                            continue;
                                        }
                                    };

                                    // Get index settings.
                                    let index_settings = match metadata.index_settings() {
                                        Ok(index_settings) => index_settings,
                                        Err(error) => {
                                            error!(?index_name, ?shard.id, ?error, "Failed to get index settings.");
                                            continue;
                                        }
                                    };

                                    // Get analyzers.
                                    let analyzers = match metadata.analyzers() {
                                        Ok(analyzers) => analyzers,
                                        Err(error) => {
                                            error!(?index_name, ?shard.id, ?error, "Failed to get analyzers.");
                                            continue;
                                        }
                                    };

                                    // Create index.
                                    info!(?shard_dir, "Creating shard index.");
                                    match create_index(
                                        &shard_dir,
                                        &schema,
                                        &index_settings,
                                        &analyzers,
                                    )
                                    .await
                                    {
                                        Ok(index) => index,
                                        Err(error) => {
                                            error!(
                                                ?shard_dir,
                                                ?error,
                                                "Failed to create shard index."
                                            );
                                            continue;
                                        }
                                    }
                                };

                                // Add index object to indices.
                                info!(?index_name, shard_id = ?shard.id, "Insert assigned index.");
                                indices
                                    .entry(index_name.clone())
                                    .or_insert_with(DashMap::new)
                                    .insert(shard.id.clone(), index.clone());

                                index
                            };

                            // Check index writer existence.
                            if index_writers
                                .entry(index_name.clone())
                                .or_insert_with(DashMap::new)
                                .contains_key(&shard.id)
                            {
                                debug!(?index_name, shard_id = ?shard.id, "Index writer already exists.");
                            } else {
                                let metadatas = metastore.metadatas().await;
                                let metadata = match metadatas.get(index_name) {
                                    Some(metadata) => metadata,
                                    None => {
                                        error!(?index_name, "Index metadata not found.");
                                        continue;
                                    }
                                };
                                let writer_threads = match metadata.writer_threads() {
                                    Ok(writer_threads) => writer_threads,
                                    Err(error) => {
                                        error!(
                                            ?index_name,
                                            ?error,
                                            "Number of index writer threads not found in metadata."
                                        );
                                        continue;
                                    }
                                };
                                let writer_mem_size = match metadata.writer_mem_size() {
                                    Ok(writer_mem_size) => writer_mem_size,
                                    Err(error) => {
                                        error!(
                                            ?index_name,
                                            ?error,
                                            "Index writer memory size not found in metadata."
                                        );
                                        continue;
                                    }
                                };

                                let index_writer = match index
                                    .writer_with_num_threads(writer_threads, writer_mem_size)
                                {
                                    Ok(index_writer) => {
                                        info!(?index_name, shard_id = ?shard.id, "Opened index writer.");
                                        index_writer
                                    }
                                    Err(error) => {
                                        error!(
                                            ?index_name,
                                            shard_id = ?shard.id,
                                            ?error,
                                            "Failed to open index writer."
                                        );
                                        continue;
                                    }
                                };

                                // Add index writer to index writers.
                                info!(?index_name, shard_id = ?shard.id, "Insert assigned index writer.");
                                index_writers
                                    .entry(index_name.clone())
                                    .or_insert_with(DashMap::new)
                                    .insert(shard.id.clone(), index_writer);
                            }

                            // Check index reader existence.
                            if index_readers
                                .entry(index_name.clone())
                                .or_insert_with(DashMap::new)
                                .contains_key(&shard.id)
                            {
                                debug!(?index_name, shard_id = ?shard.id, "Index reader already exists.");
                            } else {
                                let index_reader = match index.reader() {
                                    Ok(index_reader) => {
                                        info!(
                                            ?index_name,
                                            shard_id = ?shard.id,
                                            "Opened index reader."
                                        );
                                        index_reader
                                    }
                                    Err(error) => {
                                        error!(
                                            ?index_name,
                                            shard_id = ?shard.id,
                                            ?error,
                                            "Failed to open index reader."
                                        );
                                        continue;
                                    }
                                };

                                // Add index reader to index readers.
                                info!(?index_name, shard_id = ?shard.id, "Insert assigned index reader.");
                                index_readers
                                    .entry(index_name.clone())
                                    .or_insert_with(DashMap::new)
                                    .insert(shard.id.clone(), index_reader);
                            }
                        } else {
                            info!(?index_name, shard_id = ?shard.id, "Shard is not assigned to this node.");

                            // Remove unassigned index writer.
                            info!(?index_name, shard_id = ?shard.id, "Remove unassigned index writer.");
                            index_writers
                                .entry(index_name.clone())
                                .or_insert_with(DashMap::new)
                                .remove(&shard.id);

                            // Remove unassigned index reader.
                            info!(?index_name, shard_id = ?shard.id, "Remove unassigned index reader.");
                            index_readers
                                .entry(index_name.clone())
                                .or_insert_with(DashMap::new)
                                .remove(&shard.id);

                            // Remove unassigned index.
                            info!(?index_name, shard_id = ?shard.id, "Remove unassigned index.");
                            indices
                                .entry(index_name.clone())
                                .or_insert_with(DashMap::new)
                                .remove(&shard.id);
                        }
                    }

                    // Remove index writers for shards that no longer exist.
                    let writer_shard_ids = index_writers
                        .entry(index_name.clone())
                        .or_insert_with(DashMap::new)
                        .iter()
                        .map(|item| item.key().to_string())
                        .collect::<Vec<String>>();
                    for shard_id in writer_shard_ids.iter() {
                        if !shards.contains(shard_id) {
                            match index_writers.get_mut(index_name) {
                                Some(shard_writers) => {
                                    info!(
                                        ?index_name,
                                        ?shard_id,
                                        "Remove index writer for shard that no longer exist."
                                    );
                                    shard_writers.remove(shard_id);
                                }
                                None => {
                                    error!(
                                        ?index_name,
                                        ?shard_id,
                                        "Failed to get index writer for shard."
                                    );
                                    continue;
                                }
                            }
                        }
                    }

                    // Remove index readers for shards that no longer exist.
                    let reader_shard_ids = index_readers
                        .entry(index_name.clone())
                        .or_insert_with(DashMap::new)
                        .iter()
                        .map(|item| item.key().to_string())
                        .collect::<Vec<String>>();
                    for shard_id in reader_shard_ids.iter() {
                        if !shards.contains(shard_id) {
                            match index_readers.get_mut(index_name) {
                                Some(shard_readers) => {
                                    info!(
                                        ?index_name,
                                        ?shard_id,
                                        "Remove index reader for shard that no longer exist."
                                    );
                                    shard_readers.remove(shard_id);
                                }
                                None => {
                                    error!(
                                        ?index_name,
                                        ?shard_id,
                                        "Failed to get index reader for shard."
                                    );
                                    continue;
                                }
                            }
                        }
                    }

                    // Remove indices for shards that no longer exist.
                    let index_shard_ids = indices
                        .entry(index_name.clone())
                        .or_insert_with(DashMap::new)
                        .iter()
                        .map(|item| item.key().to_string())
                        .collect::<Vec<String>>();
                    for shard_id in index_shard_ids.iter() {
                        if !shards.contains(shard_id) {
                            match indices.get_mut(index_name) {
                                Some(shard_indices) => {
                                    info!(
                                        ?index_name,
                                        ?shard_id,
                                        "Remove index for shard that no longer exist."
                                    );
                                    shard_indices.remove(shard_id);

                                    // Remove index from file-system.
                                    let shard_dir = indices_dir
                                        .join(index_name)
                                        .join(SHARDS_DIR)
                                        .join(shard_id);
                                    info!(
                                        ?shard_dir,
                                        "Remove index for shard that no longer exist from the file system."
                                    );
                                    match fs::remove_dir_all(&shard_dir).await {
                                        Ok(_) => (),
                                        Err(error) => error!(
                                            ?shard_dir,
                                            ?error,
                                            "Failed to delete directory."
                                        ),
                                    }
                                }
                                None => {
                                    error!(
                                        ?index_name,
                                        ?shard_id,
                                        "Failed to get index for shard."
                                    );
                                    continue;
                                }
                            }
                        }
                    }

                    // Remove shard directories that exist on the file system but not in the metadata.
                    let shards_dir = indices_dir.join(index_name).join(SHARDS_DIR);
                    let mut read_dir = match fs::read_dir(&shards_dir).await {
                        Ok(dir) => dir,
                        Err(err) => {
                            debug!(?index_name, ?err, "Skip to remove deleted index shards.");
                            continue;
                        }
                    };
                    while let Ok(Some(dir_entry)) = read_dir.next_entry().await {
                        let shard_dir = dir_entry.path();
                        if !shard_dir.is_dir() {
                            // Skip non-directory entries.
                            continue;
                        }

                        let shard_id = match shard_dir.file_name() {
                            Some(name) => name.to_string_lossy().into_owned(),
                            None => {
                                warn!(?shard_dir, "Shard directory has no name.");
                                continue;
                            }
                        };

                        let shards = match metadata.shards() {
                            Ok(shards) => shards,
                            Err(error) => {
                                warn!(?index_name, ?shard_id, ?error, "Failed to get shards.");
                                continue;
                            }
                        };

                        if !shards.contains(&shard_id) {
                            info!(
                                ?index_name,
                                ?shard_id,
                                "Remove shard directory that exist on the file system but not in the metadata."
                            );
                            match fs::remove_dir_all(&shard_dir).await {
                                Ok(_) => (),
                                Err(error) => {
                                    error!(?shard_dir, ?error, "Failed to delete directory.")
                                }
                            }
                        }
                    }
                }

                // Remove index directories that do not exist in the metadatas.
                let mut indices_read_dir = match fs::read_dir(indices_dir.as_ref()).await {
                    Ok(dir) => dir,
                    Err(err) => {
                        warn!(?indices_dir, ?err, "Failed to read directory.");
                        continue;
                    }
                };
                while let Ok(Some(index_dir_entry)) = indices_read_dir.next_entry().await {
                    let index_dir = index_dir_entry.path();
                    if !index_dir.is_dir() {
                        // Skip non-directory entries.
                        continue;
                    }

                    let index_name = match index_dir.file_name() {
                        Some(name) => name.to_string_lossy().into_owned(),
                        None => {
                            warn!(?index_dir, "Index directory has no name.");
                            continue;
                        }
                    };

                    if !metadatas.contains_key(&index_name) {
                        let shards_dir = index_dir.join(SHARDS_DIR);
                        let mut shards_read_dir = match fs::read_dir(shards_dir).await {
                            Ok(dir) => dir,
                            Err(err) => {
                                warn!(?index_name, ?err, "Failed to read directory.");
                                continue;
                            }
                        };
                        while let Ok(Some(shard_dir_entry)) = shards_read_dir.next_entry().await {
                            let shard_dir = shard_dir_entry.path();
                            if !shard_dir.is_dir() {
                                // Skip non-directory entries.
                                continue;
                            }

                            let shard_id = match shard_dir.file_name() {
                                Some(name) => name.to_string_lossy().into_owned(),
                                None => {
                                    warn!(?shard_dir, "Shard directory has no name.");
                                    continue;
                                }
                            };

                            // Remove index writer that do not exist in the metadatas.
                            info!(
                                ?index_name,
                                ?shard_id,
                                "Remove index writer that does not exist in the metadatas."
                            );
                            index_writers
                                .entry(index_name.clone())
                                .or_insert_with(DashMap::new)
                                .remove(&shard_id);

                            // Remove index readers that do not exist in the metadatas.
                            info!(
                                ?index_name,
                                ?shard_id,
                                "Remove index reader that does not exist in the metadatas."
                            );
                            index_readers
                                .entry(index_name.clone())
                                .or_insert_with(DashMap::new)
                                .remove(&shard_id);

                            // Remove indices that do not exist in the metadatas.
                            info!(
                                ?index_name,
                                ?shard_id,
                                "Remove index that does not exist in the metadatas."
                            );
                            indices
                                .entry(index_name.clone())
                                .or_insert_with(DashMap::new)
                                .remove(&shard_id);
                        }

                        info!(
                            ?index_dir,
                            "Remove index directory that does not exist in the metadatas."
                        );
                        match delete_index(&index_dir).await {
                            Ok(_) => (),
                            Err(error) => error!(?index_dir, ?error, "Failed to delete directory."),
                        }
                    }
                }
            }
        });
    }

    async fn handle_messages(&self) {
        let mut receiver = self.membership.watch_message();
        let indices_dir = self.indices_dir.clone();

        tokio::spawn(async move {
            while let Some(message) = receiver.next().await {
                let kind = match message.kind() {
                    Ok(kind) => kind,
                    Err(error) => {
                        error!(?error, "Failed to get message kind.");
                        continue;
                    }
                };

                info!(?kind, "Receive message.");
                match kind {
                    MessageKind::CreateIndex => {
                        // Settings are extracted from the received messages and written to a file.
                        // No actual creating an index is performed.
                        let req = match serde_json::from_slice::<CreateIndexMessage>(message.body())
                        {
                            Ok(req) => req,
                            Err(error) => {
                                error!(?error, "Failed to deserialize request.");
                                continue;
                            }
                        };

                        // Create the index directory
                        let index_dir = indices_dir.join(&req.name);
                        if index_dir.exists() {
                            warn!(?index_dir, "Index directory already exists.");
                        } else {
                            match fs::create_dir_all(&index_dir).await {
                                Ok(_) => info!(?index_dir, "Index directory has been created."),
                                Err(error) => {
                                    error!(?index_dir, ?error, "Failed to create index directory.");
                                    continue;
                                }
                            }
                        }

                        // Save meta.json
                        let meta_path = index_dir.join(INDEX_METADATA_FILE);
                        if meta_path.exists() {
                            warn!(?meta_path, "File already exists.");
                        } else {
                            match save_index_metadata(&meta_path, req.meta.clone()).await {
                                Ok(_) => info!(?meta_path, "File have been saved."),
                                Err(error) => {
                                    error!(?meta_path, ?error, "Failed to write file.");
                                    continue;
                                }
                            }
                        }
                    }
                    MessageKind::DeleteIndex => {
                        let req = match serde_json::from_slice::<DeleteIndexMessage>(message.body())
                        {
                            Ok(req) => req,
                            Err(error) => {
                                error!(?error, "Failed to deserialize request.");
                                continue;
                            }
                        };

                        // Create the index directory
                        let index_dir = indices_dir.join(&req.name);

                        // Delete index metadata file.
                        let index_metadata_path = index_dir.join(INDEX_METADATA_FILE);
                        if index_metadata_path.exists() {
                            match remove_file(&index_metadata_path).await {
                                Ok(_) => {
                                    info!(
                                        ?index_metadata_path,
                                        "Index config file have been deleted."
                                    )
                                }
                                Err(error) => {
                                    error!(
                                        ?index_metadata_path,
                                        ?error,
                                        "Failed to delete index config file."
                                    );
                                }
                            }
                        }
                    }
                    MessageKind::ModifyIndex => {
                        let req = match serde_json::from_slice::<ModifyIndexMessage>(message.body())
                        {
                            Ok(req) => req,
                            Err(error) => {
                                error!(?error, "Failed to deserialize request.");
                                continue;
                            }
                        };

                        let index_dir = indices_dir.join(req.name.as_str());

                        // Check index directory existence.
                        if !index_dir.exists() {
                            warn!(?index_dir, "Index directory does not exists.");
                            continue;
                        }

                        // Save shards.json
                        match save_index_metadata(
                            &index_dir.join(INDEX_METADATA_FILE),
                            req.index_metadata.clone(),
                        )
                        .await
                        {
                            Ok(_) => {}
                            Err(error) => {
                                error!(?error, "Failed to update index config.");
                                continue;
                            }
                        }
                    }
                    _ => {
                        debug!(?kind, "Unhandled message kind.");
                        continue;
                    }
                }
            }
        });
    }

    pub fn members_watcher(&self) -> WatchStream<Members> {
        self.membership.watch_members()
    }

    pub async fn local_member(&self) -> Member {
        self.membership.local_member().await
    }

    pub async fn remote_members(&self) -> Vec<Member> {
        self.membership.remote_members().await
    }

    pub async fn members(&self) -> Vec<Member> {
        self.membership.members().await.iter().cloned().collect()
    }

    pub async fn index_metadata(&self, name: &str) -> Option<IndexMetadata> {
        self.metastore.metadatas().await.get(name).cloned()
    }

    pub async fn shards(&self, index_name: &str) -> Option<Shards> {
        self.index_metadata(index_name).await?.shards().ok()
    }

    pub async fn assign_shard(&self, index_name: &str, doc_id: &str) -> Result<Shard, NodeError> {
        let shards = self.shards(index_name).await.ok_or_else(|| {
            NodeErrorKind::ShardNotFound
                .with_error(anyhow::anyhow!("Index {} not found.", index_name))
        })?;
        let shard = shards.lookup_shard(doc_id).ok_or_else(|| {
            NodeErrorKind::ShardNotFound.with_error(anyhow::anyhow!("Key {} not found.", doc_id))
        })?;

        Ok(shard.clone())
    }

    pub async fn create_index(
        &self,
        request: CreateIndexRequest,
    ) -> Result<CreateIndexResponse, NodeError> {
        let meta = serde_json::from_slice::<IndexMetadata>(&request.metadata)
            .map_err(|error| NodeErrorKind::MetadataError.with_error(error))?;

        let kind = MessageKind::CreateIndex;
        let message = CreateIndexMessage {
            name: request.name,
            meta,
        };
        let body = serde_json::to_vec(&message)
            .map_err(|error| NodeErrorKind::MessageSerializationFailure.with_error(error))?;
        let version = OffsetDateTime::now_utc().unix_timestamp();

        match self
            .membership
            .broadcast(Message::with_body_version(kind, body.as_slice(), version))
            .await
        {
            Ok(_) => Ok(CreateIndexResponse {}),
            Err(error) => Err(NodeErrorKind::IndexCreationFailure.with_error(error)),
        }
    }

    pub async fn delete_index(
        &self,
        request: DeleteIndexRequest,
    ) -> Result<DeleteIndexResponse, NodeError> {
        let kind = MessageKind::DeleteIndex;
        let message = DeleteIndexMessage { name: request.name };
        let body = serde_json::to_vec(&message)
            .map_err(|error| NodeErrorKind::MessageSerializationFailure.with_error(error))?;
        let version = OffsetDateTime::now_utc().unix_timestamp();

        match self
            .membership
            .broadcast(Message::with_body_version(kind, body.as_slice(), version))
            .await
        {
            Ok(_) => Ok(DeleteIndexResponse {}),
            Err(error) => Err(NodeErrorKind::IndexDeletionFailure.with_error(error)),
        }
    }

    pub async fn get_index(&self, request: GetIndexRequest) -> Result<GetIndexResponse, NodeError> {
        let metadata_path = self
            .indices_dir
            .join(request.name)
            .join(INDEX_METADATA_FILE);

        // Read raw index metadata file.
        let metadata_bytes = read_file(&metadata_path)
            .await
            .map_err(|error| NodeErrorKind::FileReadFailure.with_error(error))?;

        Ok(GetIndexResponse {
            metadata: metadata_bytes,
        })
    }

    pub async fn modify_index(
        &self,
        request: ModifyIndexRequest,
    ) -> Result<ModifyIndexResponse, NodeError> {
        let mut index_metadata = self.index_metadata(&request.name).await.ok_or_else(|| {
            NodeErrorKind::IndexConfigDoesNotExist.with_error(anyhow::anyhow!(
                "Index config for {} does not exist.",
                request.name
            ))
        })?;

        let new_index_metadata = serde_json::from_slice::<IndexMetadata>(&request.metadata)
            .map_err(|error| NodeErrorKind::MetadataError.with_error(error))?;

        let mut changed = false;

        let writer_threads = index_metadata
            .writer_threads()
            .map_err(|error| NodeErrorKind::MetadataError.with_error(error))?;
        let new_writer_threads = new_index_metadata
            .writer_threads()
            .map_err(|error| NodeErrorKind::MetadataError.with_error(error))?;
        if writer_threads != new_writer_threads {
            index_metadata
                .set_writer_threads(new_writer_threads)
                .map_err(|error| NodeErrorKind::MetadataError.with_error(error))?;
            changed = true;
        }

        let writer_mem_size = index_metadata
            .writer_mem_size()
            .map_err(|error| NodeErrorKind::MetadataError.with_error(error))?;
        let new_writer_mem_size = new_index_metadata
            .writer_mem_size()
            .map_err(|error| NodeErrorKind::MetadataError.with_error(error))?;
        if writer_mem_size != new_writer_mem_size {
            index_metadata
                .set_writer_mem_size(new_writer_mem_size)
                .map_err(|error| NodeErrorKind::MetadataError.with_error(error))?;
            changed = true;
        }

        let index_settings = index_metadata
            .index_settings()
            .map_err(|error| NodeErrorKind::MetadataError.with_error(error))?;
        let new_index_settings = new_index_metadata
            .index_settings()
            .map_err(|error| NodeErrorKind::MetadataError.with_error(error))?;
        if index_settings != new_index_settings {
            warn!("Sorry. We don't support changing index settings at this time.");
        }

        let num_replicas = index_metadata
            .num_replicas()
            .map_err(|error| NodeErrorKind::MetadataError.with_error(error))?;
        let new_num_replicas = new_index_metadata
            .num_replicas()
            .map_err(|error| NodeErrorKind::MetadataError.with_error(error))?;
        if num_replicas != new_num_replicas {
            index_metadata
                .set_num_replicas(new_num_replicas)
                .map_err(|error| NodeErrorKind::MetadataError.with_error(error))?;
            changed = true;
        }

        let num_shards = index_metadata
            .num_shards()
            .map_err(|error| NodeErrorKind::MetadataError.with_error(error))?;
        let new_num_shards = new_index_metadata
            .num_shards()
            .map_err(|error| NodeErrorKind::MetadataError.with_error(error))?;
        if num_shards != new_num_shards {
            index_metadata
                .set_num_shards(new_num_shards)
                .map_err(|error| NodeErrorKind::MetadataError.with_error(error))?;
            changed = true;
        }

        if !changed {
            warn!("There is no change in the index config.");
            return Ok(ModifyIndexResponse {});
        }

        let kind = MessageKind::ModifyIndex;
        let message = ModifyIndexMessage {
            name: request.name,
            index_metadata: index_metadata.clone(),
        };
        let body = serde_json::to_vec(&message)
            .map_err(|error| NodeErrorKind::MessageSerializationFailure.with_error(error))?;
        let version = OffsetDateTime::now_utc().unix_timestamp();

        match self
            .membership
            .broadcast(Message::with_body_version(kind, body.as_slice(), version))
            .await
        {
            Ok(_) => Ok(ModifyIndexResponse {}),
            Err(error) => Err(NodeErrorKind::ModifyIndexFailure.with_error(error)),
        }
    }

    pub async fn put_documents(
        &self,
        request: PutDocumentsRequest,
    ) -> Result<PutDocumentsResponse, NodeError> {
        // Index.
        let shards = self.indices.get(&request.name).ok_or_else(|| {
            NodeErrorKind::IndexNotFound
                .with_error(anyhow::anyhow!("{} not found in indices.", &request.name))
        })?;
        let shard = shards.get(&request.shard_id).ok_or_else(|| {
            NodeErrorKind::ShardNotFound.with_error(anyhow::anyhow!(
                "{} not found in shards.",
                &request.shard_id
            ))
        })?;

        // Schema.
        let schema = shard.schema();

        // Document ID field.
        let doc_id_field = schema.get_field(DOC_ID_FIELD_NAME).ok_or_else(|| {
            NodeErrorKind::FieldNotFound
                .with_error(anyhow::anyhow!("Field {:?} not found.", DOC_ID_FIELD_NAME))
        })?;

        // Document timestamp field.
        let doc_timestamp_field = schema.get_field(DOC_TIMESTAMP_FIELD_NAME).ok_or_else(|| {
            NodeErrorKind::FieldNotFound.with_error(anyhow::anyhow!(
                "Field {:?} not found.",
                DOC_TIMESTAMP_FIELD_NAME
            ))
        })?;

        // Index writer.
        let shard_indexers = self.index_writers.get(&request.name).ok_or_else(|| {
            NodeErrorKind::IndexNotFound
                .with_error(anyhow::anyhow!("Indexer not found for {}.", &request.name))
        })?;
        let index_writer = shard_indexers.get(&request.shard_id).ok_or_else(|| {
            NodeErrorKind::IndexNotFound.with_error(anyhow::anyhow!(
                "Indexer not found for {}.",
                &request.shard_id
            ))
        })?;

        for doc_bytes in request.docs {
            let doc_value = serde_json::from_slice::<serde_json::Value>(doc_bytes.as_slice())
                .map_err(|error| NodeErrorKind::DocumentDeserializationFailure.with_error(error))?;

            let doc_value_map = match doc_value.as_object() {
                Some(doc_map) => doc_map,
                None => {
                    error!("Document is not a map.");
                    continue;
                }
            };

            let doc_id = doc_value_map["id"].as_str().ok_or_else(|| {
                NodeErrorKind::DocumentDeserializationFailure
                    .with_error(anyhow::anyhow!("Document ID not found."))
            })?;

            let doc_fields = doc_value_map["fields"].as_object().ok_or_else(|| {
                NodeErrorKind::DocumentDeserializationFailure
                    .with_error(anyhow::anyhow!("Document fields not found."))
            })?;
            let doc_fields_json = serde_json::to_string(&doc_fields)
                .map_err(|error| NodeErrorKind::DocumentSerializationFailure.with_error(error))?;

            // Deserialize the JSON to the document.
            let mut doc = schema
                .parse_document(&doc_fields_json)
                .map_err(|error| NodeErrorKind::DocumentParseFailure.with_error(error))?;

            // Add the document ID.
            doc.add_text(doc_id_field, doc_id);

            // Add the document ID.
            doc.add_date(
                doc_timestamp_field,
                DateTime::from_utc(OffsetDateTime::now_utc()),
            );

            let _opstamp = index_writer.delete_term(Term::from_field_text(doc_id_field, doc_id));
            let _opstamp = index_writer.add_document(doc);
        }

        Ok(PutDocumentsResponse {})
    }

    pub async fn delete_documents(
        &self,
        request: DeleteDocumentsRequest,
    ) -> Result<DeleteDocumentsResponse, NodeError> {
        // Index.
        let shards = self.indices.get(&request.name).ok_or_else(|| {
            NodeErrorKind::IndexNotFound
                .with_error(anyhow::anyhow!("{} not found in indices.", &request.name))
        })?;
        let shard = shards.get(&request.shard_id).ok_or_else(|| {
            NodeErrorKind::ShardNotFound.with_error(anyhow::anyhow!(
                "{} not found in shards.",
                &request.shard_id
            ))
        })?;

        // Schema.
        let schema = shard.schema();

        // Document ID field.
        let doc_id_field = schema.get_field(DOC_ID_FIELD_NAME).ok_or_else(|| {
            NodeErrorKind::FieldNotFound
                .with_error(anyhow::anyhow!("Field {:?} not found.", DOC_ID_FIELD_NAME))
        })?;

        // Index writer.
        let shard_indexers = self.index_writers.get(&request.name).ok_or_else(|| {
            NodeErrorKind::IndexNotFound
                .with_error(anyhow::anyhow!("Indexer not found for {}.", &request.name))
        })?;
        let index_writer = shard_indexers.get(&request.shard_id).ok_or_else(|| {
            NodeErrorKind::IndexNotFound.with_error(anyhow::anyhow!(
                "Indexer not found for {}.",
                &request.shard_id
            ))
        })?;

        for doc_id in request.doc_ids {
            let term = Term::from_field_text(doc_id_field, &doc_id);
            let _opstamp = index_writer.delete_term(term);
        }

        Ok(DeleteDocumentsResponse {})
    }

    pub async fn commit(&self, request: CommitRequest) -> Result<CommitResponse, NodeError> {
        // Index writer.
        let shard_indexers = self.index_writers.get(&request.name).ok_or_else(|| {
            NodeErrorKind::IndexNotFound
                .with_error(anyhow::anyhow!("Indexer not found for {}.", &request.name))
        })?;
        let mut index_writer = shard_indexers.get_mut(&request.shard_id).ok_or_else(|| {
            NodeErrorKind::IndexNotFound.with_error(anyhow::anyhow!(
                "Indexer not found for {}.",
                &request.shard_id
            ))
        })?;

        let _opstamp = index_writer
            .commit()
            .map_err(|error| NodeErrorKind::IndexCommitFailure.with_error(error))?;

        Ok(CommitResponse {})
    }

    pub async fn rollback(&self, request: RollbackRequest) -> Result<RollbackResponse, NodeError> {
        // Index writer.
        let shard_indexers = self.index_writers.get(&request.name).ok_or_else(|| {
            NodeErrorKind::IndexNotFound
                .with_error(anyhow::anyhow!("Writer not found for {}.", &request.name))
        })?;
        let mut index_writer = shard_indexers.get_mut(&request.shard_id).ok_or_else(|| {
            NodeErrorKind::IndexNotFound.with_error(anyhow::anyhow!(
                "Writer not found for {}.",
                &request.shard_id
            ))
        })?;

        let _opstamp = index_writer
            .rollback()
            .map_err(|error| NodeErrorKind::IndexRollbackFailure.with_error(error))?;

        Ok(RollbackResponse {})
    }

    pub async fn search(&self, request: SearchRequest) -> Result<SearchResponse, NodeError> {
        // Get searcher.
        let index_readers = self.index_readers.get(&request.name).ok_or_else(|| {
            NodeErrorKind::IndexNotFound
                .with_error(anyhow::anyhow!("Reader not found for {}.", &request.name))
        })?;
        let index_reader = index_readers.get(&request.shard_id).ok_or_else(|| {
            NodeErrorKind::IndexNotFound.with_error(anyhow::anyhow!(
                "Reader not found for {}.",
                &request.shard_id
            ))
        })?;
        let searcher = index_reader.searcher();

        // Get schema.
        let schema = searcher.schema();

        // Make query.
        let query = request.query.ok_or_else(|| {
            NodeErrorKind::SearchFailre.with_error(anyhow::anyhow!("Query not found."))
        })?;
        let query = create_query(&query, &searcher)
            .map_err(|error| NodeErrorKind::SearchFailre.with_error(error))?;

        // Get collection kind.
        let kind = CollectionKind::from_i32(request.collection_kind).ok_or_else(|| {
            NodeErrorKind::SearchFailre.with_error(anyhow::anyhow!("Collection kind not found."))
        })?;

        let result = match request.sort {
            Some(sort) => {
                // Sort field.
                let sort_field = schema.get_field(&sort.field).ok_or_else(|| {
                    NodeErrorKind::IndexSearchFailure.with_error(anyhow::anyhow!(
                        "Sort field {:?} does not exist.",
                        sort.field
                    ))
                })?;

                // Sort field type.
                let sort_field_type = schema.get_field_entry(sort_field).field_type();

                // Is fast field?
                if !sort_field_type.is_fast() {
                    return Err(NodeErrorKind::SearchFailre.with_error(anyhow::anyhow!(
                        "Sort fields {:?} must be fast field.",
                        sort_field_type,
                    )));
                }

                // Sort order.
                let sort_order = Order::from_i32(sort.order).ok_or_else(|| {
                    NodeErrorKind::SearchFailre.with_error(anyhow::anyhow!(
                        "Sort order {:?} is not supported.",
                        sort.order
                    ))
                })?;

                let (total_hits, documents) = match (sort_field_type, sort_order) {
                    (FieldType::I64(_field_opts), Order::Asc) => search_order_by_i64_asc(
                        &searcher,
                        query,
                        request.hits,
                        request.offset,
                        sort,
                        request.fields,
                        kind,
                    )?,
                    (FieldType::I64(_field_opts), Order::Desc) => search_order_by_i64_desc(
                        &searcher,
                        query,
                        request.hits,
                        request.offset,
                        sort,
                        request.fields,
                        kind,
                    )?,
                    (FieldType::F64(_field_opts), Order::Asc) => search_order_by_f64_asc(
                        &searcher,
                        query,
                        request.hits,
                        request.offset,
                        sort,
                        request.fields,
                        kind,
                    )?,
                    (FieldType::F64(_field_opts), Order::Desc) => search_order_by_f64_desc(
                        &searcher,
                        query,
                        request.hits,
                        request.offset,
                        sort,
                        request.fields,
                        kind,
                    )?,
                    (FieldType::U64(_field_opts), Order::Asc) => search_order_by_u64_asc(
                        &searcher,
                        query,
                        request.hits,
                        request.offset,
                        sort,
                        request.fields,
                        kind,
                    )?,
                    (FieldType::U64(_field_opts), Order::Desc) => search_order_by_u64_desc(
                        &searcher,
                        query,
                        request.hits,
                        request.offset,
                        sort,
                        request.fields,
                        kind,
                    )?,
                    (FieldType::Date(_field_opts), Order::Asc) => search_order_by_date_asc(
                        &searcher,
                        query,
                        request.hits,
                        request.offset,
                        sort,
                        request.fields,
                        kind,
                    )?,
                    (FieldType::Date(_field_opts), Order::Desc) => search_order_by_date_desc(
                        &searcher,
                        query,
                        request.hits,
                        request.offset,
                        sort,
                        request.fields,
                        kind,
                    )?,
                    _ => {
                        return Err(NodeErrorKind::SearchFailre.with_error(anyhow::anyhow!(
                            "Sort field {:?} is not supported.",
                            sort.field
                        )));
                    }
                };

                SearchResponse {
                    total_hits,
                    documents,
                }
            }
            None => {
                let (total_hits, documents) = search_order_by_score_desc(
                    &searcher,
                    query,
                    request.hits,
                    request.offset,
                    request.fields,
                    kind,
                )?;

                SearchResponse {
                    total_hits,
                    documents,
                }
            }
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::node::index::extract_index_name;
    use std::path::Path;

    #[test]
    fn test_extract_index_name() {
        let path = Path::new("/tmp/indices/test_index");
        let index_name = extract_index_name(path).unwrap();
        assert_eq!(&index_name, "test_index");

        let path = Path::new("/tmp/indices/test_index/");
        let index_name = extract_index_name(path).unwrap();
        assert_eq!(&index_name, "test_index");

        let path = Path::new("/tmp/indices/test_index/shards");
        let index_name = extract_index_name(path).unwrap();
        assert_eq!(&index_name, "test_index");

        let path = Path::new("/tmp/indices/test_index/shards/");
        let index_name = extract_index_name(path).unwrap();
        assert_eq!(&index_name, "test_index");

        let path = Path::new("/tmp/indices/test_index/shards/test_shard");
        let index_name = extract_index_name(path).unwrap();
        assert_eq!(&index_name, "test_index");

        let path = Path::new("/tmp/indices/test_index/shards/test_shard/");
        let index_name = extract_index_name(path).unwrap();
        assert_eq!(&index_name, "test_index");

        let path = Path::new("/tmp/indices/test_index/shards/test_shard/meta.json");
        let index_name = extract_index_name(path).unwrap();
        assert_eq!(&index_name, "test_index");

        let path = Path::new("/tmp/tmp2/indices/test_index");
        let index_name = extract_index_name(path).unwrap();
        assert_eq!(&index_name, "test_index");

        let path = Path::new("/tmp/tmp2/indices/test_index/");
        let index_name = extract_index_name(path).unwrap();
        assert_eq!(&index_name, "test_index");

        let path = Path::new("/tmp/tmp2/indices/test_index/shards");
        let index_name = extract_index_name(path).unwrap();
        assert_eq!(&index_name, "test_index");

        let path = Path::new("/tmp/tmp2/indices/test_index/shards/");
        let index_name = extract_index_name(path).unwrap();
        assert_eq!(&index_name, "test_index");

        let path = Path::new("/tmp/tmp2/indices/test_index/shards/test_shard");
        let index_name = extract_index_name(path).unwrap();
        assert_eq!(&index_name, "test_index");

        let path = Path::new("/tmp/tmp2/indices/test_index/shards/test_shard/");
        let index_name = extract_index_name(path).unwrap();
        assert_eq!(&index_name, "test_index");

        let path = Path::new("/tmp/tmp2/indices/test_index/shards/test_shard/meta.json");
        let index_name = extract_index_name(path).unwrap();
        assert_eq!(&index_name, "test_index");

        let path = Path::new("/indices/test_index");
        let index_name = extract_index_name(path).unwrap();
        assert_eq!(&index_name, "test_index");

        let path = Path::new("/indices/test_index/");
        let index_name = extract_index_name(path).unwrap();
        assert_eq!(&index_name, "test_index");

        let path = Path::new("/indices/test_index/shards");
        let index_name = extract_index_name(path).unwrap();
        assert_eq!(&index_name, "test_index");

        let path = Path::new("/indices/test_index/shards/");
        let index_name = extract_index_name(path).unwrap();
        assert_eq!(&index_name, "test_index");

        let path = Path::new("/indices/test_index/shards/test_shard");
        let index_name = extract_index_name(path).unwrap();
        assert_eq!(&index_name, "test_index");

        let path = Path::new("/indices/test_index/shards/test_shard/");
        let index_name = extract_index_name(path).unwrap();
        assert_eq!(&index_name, "test_index");

        let path = Path::new("/indices/test_index/shards/test_shard/meta.json");
        let index_name = extract_index_name(path).unwrap();
        assert_eq!(&index_name, "test_index");
    }
}
