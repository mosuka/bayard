use std::{collections::HashMap, fmt, path::PathBuf, sync::Arc};

use crossbeam::channel::{unbounded, TryRecvError};
use notify::{
    event::{DataChange, ModifyKind, RemoveKind, RenameMode},
    EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use once_cell::sync::Lazy;
use regex::Regex;
use tokio::{
    fs,
    sync::{watch, RwLock},
    time::{sleep, Duration},
};
use tokio_stream::wrappers::WatchStream;
use tracing::{debug, error, info};

use crate::{
    index::{metadata::load_index_metadata, INDEX_METADATA_FILE},
    node::index::extract_index_name,
};

use super::{metadata::Metadata, INDEX_METADATA_FILE_PATTERN};

const SLEEP_DURATION: Duration = Duration::from_millis(100);

static INDEX_METADATA_FILE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(INDEX_METADATA_FILE_PATTERN).unwrap());

#[derive(Debug, Clone, Copy)]
pub enum MetastoreErrorKind {
    DirectoryCreationFailure,
    DirectoryReadFailure,
}

impl MetastoreErrorKind {
    pub fn with_error<E>(self, source: E) -> MetastoreError
    where
        anyhow::Error: From<E>,
    {
        MetastoreError {
            kind: self,
            source: From::from(source),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("MetastoreError(kind={kind:?}, source={source})")]
pub struct MetastoreError {
    pub kind: MetastoreErrorKind,
    #[source]
    source: anyhow::Error,
}

impl MetastoreError {
    pub fn add_context<C>(self, ctx: C) -> Self
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        MetastoreError {
            kind: self.kind,
            source: self.source.context(ctx),
        }
    }

    pub fn kind(&self) -> MetastoreErrorKind {
        self.kind
    }
}

pub struct Metastore {
    metadatas: Arc<RwLock<HashMap<String, Metadata>>>, // index name -> metadata
    metadatas_receiver: watch::Receiver<HashMap<String, Metadata>>,
}

impl Metastore {
    pub async fn new(indices_dir: PathBuf) -> Result<Self, MetastoreError> {
        if !indices_dir.exists() {
            info!(?indices_dir, "Creating indices directory.");
            fs::create_dir_all(&indices_dir)
                .await
                .map_err(|error| MetastoreErrorKind::DirectoryCreationFailure.with_error(error))?;
        }

        // Load existing index metadatas.
        let metadatas: Arc<RwLock<HashMap<String, Metadata>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let mut indices_read_dir = fs::read_dir(&indices_dir)
            .await
            .map_err(|error| MetastoreErrorKind::DirectoryReadFailure.with_error(error))?;
        while let Some(index_dir_entry) = indices_read_dir
            .next_entry()
            .await
            .map_err(|error| MetastoreErrorKind::DirectoryReadFailure.with_error(error))?
        {
            let mut mut_metadatas = metadatas.write().await;

            // Get index directory path.
            let index_dir = index_dir_entry.path();
            if !index_dir.is_dir() {
                // Skip non-directory entries.
                continue;
            }

            // Extract index name from path.
            let index_name = match extract_index_name(&index_dir) {
                Ok(index_name) => index_name,
                Err(error) => {
                    error!(?index_dir, ?error, "Failed to extract index name.");
                    continue;
                }
            };

            // Loading index metada.
            let index_metadata_path = index_dir.join(INDEX_METADATA_FILE);
            info!(?index_metadata_path, "Loading index metadata.");
            let metadata = match load_index_metadata(&index_metadata_path).await {
                Ok(index_metadata) => index_metadata,
                Err(error) => {
                    error!(
                        ?index_metadata_path,
                        ?error,
                        "Failed to load index metadata."
                    );
                    continue;
                }
            };

            mut_metadatas.insert(index_name, metadata);
        }

        let (metadatas_sender, metadatas_receiver) =
            watch::channel::<HashMap<String, Metadata>>(metadatas.read().await.clone());

        let indices_dir_task = indices_dir.clone();
        let metadatas_task = Arc::clone(&metadatas);

        tokio::spawn(async move {
            // let index_meta_re = Regex::new(INDEX_METADATA_FILE_PATTERN).unwrap();

            let (tx_filesystem, rx_filesystem) = unbounded();

            let mut watcher: RecommendedWatcher =
                RecommendedWatcher::new(tx_filesystem).expect("Watch error.");
            watcher
                .watch(indices_dir_task.as_path(), RecursiveMode::Recursive)
                .expect("Watch error.");

            loop {
                match rx_filesystem.try_recv() {
                    Ok(event) => {
                        let event = match event {
                            Ok(event) => event,
                            Err(error) => {
                                error!(?error, "Receive error event.");
                                continue;
                            }
                        };

                        let path = match event.paths.first() {
                            Some(path) => path,
                            None => {
                                error!(?event, "No path in event.");
                                continue;
                            }
                        };

                        if INDEX_METADATA_FILE_REGEX.is_match(path.to_str().unwrap_or("")) {
                            // Extract index name from path.
                            let index_name = match extract_index_name(path) {
                                Ok(name) => name,
                                Err(error) => {
                                    error!(?path, ?error, "Failed to extract index name.");
                                    continue;
                                }
                            };

                            match event.kind {
                                EventKind::Modify(ModifyKind::Name(RenameMode::To))
                                | EventKind::Modify(ModifyKind::Data(DataChange::Any)) => {
                                    // Load new metadata.
                                    info!(?path, "Load new metadata.");
                                    let new_metadata = match load_index_metadata(path).await {
                                        Ok(index_metadata) => index_metadata,
                                        Err(error) => {
                                            error!(?path, ?error, "Failed to load metadata.");
                                            continue;
                                        }
                                    };

                                    let mut mut_metadatas = metadatas_task.write().await;
                                    // Get current index metadata.
                                    info!(?index_name, "Load current metadata.");
                                    let current_metadata = match mut_metadatas.get_mut(&index_name)
                                    {
                                        Some(metadata) => metadata,
                                        None => {
                                            error!(?index_name, "No metadata found.");
                                            continue;
                                        }
                                    };

                                    // Check schema difference.
                                    let current_schema = match current_metadata.schema() {
                                        Ok(schema) => schema,
                                        Err(error) => {
                                            error!(
                                                ?index_name,
                                                ?error,
                                                "Failed to get current schema."
                                            );
                                            continue;
                                        }
                                    };
                                    let new_schema = match new_metadata.schema() {
                                        Ok(schema) => schema,
                                        Err(error) => {
                                            error!(
                                                ?index_name,
                                                ?error,
                                                "Failed to get new schema."
                                            );
                                            continue;
                                        }
                                    };
                                    if current_schema != new_schema {
                                        // Send event message.
                                        info!("Schema has changed.");
                                    }

                                    // Check analyzers difference.
                                    let current_analyzers = match current_metadata.analyzers() {
                                        Ok(analyzers) => analyzers,
                                        Err(error) => {
                                            error!(
                                                ?index_name,
                                                ?error,
                                                "Failed to get current analyzers."
                                            );
                                            continue;
                                        }
                                    };
                                    let new_analyzers = match new_metadata.analyzers() {
                                        Ok(analyzers) => analyzers,
                                        Err(error) => {
                                            error!(
                                                ?index_name,
                                                ?error,
                                                "Failed to get new analyzers."
                                            );
                                            continue;
                                        }
                                    };
                                    if current_analyzers != new_analyzers {
                                        // Send event message.
                                        info!("Analyzers has changed.");
                                    }

                                    // Check index settings difference.
                                    let current_index_settings =
                                        match current_metadata.index_settings() {
                                            Ok(index_settings) => index_settings,
                                            Err(error) => {
                                                error!(
                                                    ?index_name,
                                                    ?error,
                                                    "Failed to get current index settings."
                                                );
                                                continue;
                                            }
                                        };
                                    let new_index_settings = match new_metadata.index_settings() {
                                        Ok(index_settings) => index_settings,
                                        Err(error) => {
                                            error!(
                                                ?index_name,
                                                ?error,
                                                "Failed to get new index settings."
                                            );
                                            continue;
                                        }
                                    };
                                    if current_index_settings != new_index_settings {
                                        // Send event message.
                                        info!("Index settings has changed.");
                                    }

                                    // Check writer threads difference.
                                    let current_writer_threads =
                                        match current_metadata.writer_threads() {
                                            Ok(writer_threads) => writer_threads,
                                            Err(error) => {
                                                error!(
                                                    ?index_name,
                                                    ?error,
                                                    "Failed to get current writer threads."
                                                );
                                                continue;
                                            }
                                        };
                                    let new_writer_threads = match new_metadata.writer_threads() {
                                        Ok(writer_threads) => writer_threads,
                                        Err(error) => {
                                            error!(
                                                ?index_name,
                                                ?error,
                                                "Failed to get new writer threads."
                                            );
                                            continue;
                                        }
                                    };
                                    if current_writer_threads != new_writer_threads {
                                        // Send event message.
                                        info!("Writer threads has changed.");
                                    }

                                    // Check writer memory size difference.
                                    let current_writer_mem_size =
                                        match current_metadata.writer_mem_size() {
                                            Ok(writer_mem_size) => writer_mem_size,
                                            Err(error) => {
                                                error!(
                                                    ?index_name,
                                                    ?error,
                                                    "Failed to get current writer memory size."
                                                );
                                                continue;
                                            }
                                        };
                                    let new_writer_mem_size = match new_metadata.writer_mem_size() {
                                        Ok(writer_mem_size) => writer_mem_size,
                                        Err(error) => {
                                            error!(
                                                ?index_name,
                                                ?error,
                                                "Failed to get new writer memory size."
                                            );
                                            continue;
                                        }
                                    };
                                    if current_writer_mem_size != new_writer_mem_size {
                                        // Send event message.
                                        info!("Writer memory size has changed.");
                                    }

                                    // Check number of replicas difference.
                                    let current_num_replicas = match current_metadata.num_replicas()
                                    {
                                        Ok(replicas) => replicas,
                                        Err(error) => {
                                            error!(
                                                ?index_name,
                                                ?error,
                                                "Failed to get current number of replicas."
                                            );
                                            continue;
                                        }
                                    };
                                    let new_num_replicas = match new_metadata.num_replicas() {
                                        Ok(replicas) => replicas,
                                        Err(error) => {
                                            error!(
                                                ?index_name,
                                                ?error,
                                                "Failed to get new number of replicas."
                                            );
                                            continue;
                                        }
                                    };
                                    if current_num_replicas != new_num_replicas {
                                        // Send event message.
                                        info!("Number of replicas has changed.");
                                    }

                                    // Check number of shards difference.
                                    let current_num_shards = match current_metadata.num_shards() {
                                        Ok(shards) => shards,
                                        Err(error) => {
                                            error!(
                                                ?index_name,
                                                ?error,
                                                "Failed to get current number of shards."
                                            );
                                            continue;
                                        }
                                    };
                                    let new_num_shards = match new_metadata.num_shards() {
                                        Ok(shards) => shards,
                                        Err(error) => {
                                            error!(
                                                ?index_name,
                                                ?error,
                                                "Failed to get new number of shards."
                                            );
                                            continue;
                                        }
                                    };
                                    if current_num_shards != new_num_shards {
                                        // Send event message.
                                        info!("Number of shards has changed.");
                                    }

                                    // Check shards difference.
                                    let current_shards = match current_metadata.shards() {
                                        Ok(shards) => shards,
                                        Err(error) => {
                                            error!(
                                                ?index_name,
                                                ?error,
                                                "Failed to get current shards."
                                            );
                                            continue;
                                        }
                                    };
                                    let new_shards = match new_metadata.shards() {
                                        Ok(shards) => shards,
                                        Err(error) => {
                                            error!(
                                                ?index_name,
                                                ?error,
                                                "Failed to get new shards."
                                            );
                                            continue;
                                        }
                                    };
                                    if current_shards != new_shards {
                                        // Send event message.
                                        info!("Shards has changed.");
                                    }

                                    // Update metadatas.
                                    info!(?index_name, "Insert index metadata.");
                                    mut_metadatas.insert(index_name.clone(), new_metadata.clone());

                                    match metadatas_sender.send(mut_metadatas.clone()) {
                                        Ok(_) => info!("Sent index metadatas to the stream."),
                                        Err(_error) => {
                                            error!(
                                                ?index_name,
                                                // ?error,
                                                "Failed to send index metadatas."
                                            );
                                        }
                                    }
                                }
                                EventKind::Remove(RemoveKind::File) => {
                                    info!(?index_name, "Remove index metadata.");
                                    let mut mut_metadatas = metadatas_task.write().await;
                                    mut_metadatas.remove(&index_name);

                                    match metadatas_sender.send(mut_metadatas.clone()) {
                                        Ok(_) => info!("Sent index metadatas to the stream."),
                                        Err(_error) => {
                                            error!(
                                                ?index_name,
                                                // ?error,
                                                "Failed to send index metadatas."
                                            );
                                        }
                                    }
                                }
                                _ => {
                                    debug!(?event, "Unexpected event kind.");
                                    continue;
                                }
                            }
                        } else {
                            debug!(?path, "Ignore path.");
                            continue;
                        }
                    }
                    Err(TryRecvError::Empty) => {
                        sleep(SLEEP_DURATION).await;
                    }
                    Err(TryRecvError::Disconnected) => {
                        break;
                    }
                }
            }
        });

        Ok(Metastore {
            metadatas,
            metadatas_receiver,
        })
    }

    pub async fn metadatas(&self) -> HashMap<String, Metadata> {
        self.metadatas.read().await.clone()
    }

    pub fn watch(&self) -> WatchStream<HashMap<String, Metadata>> {
        WatchStream::new(self.metadatas_receiver.clone())
    }
}
