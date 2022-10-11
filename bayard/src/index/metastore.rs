use std::{collections::HashMap, fmt, path::PathBuf, sync::Arc};

use crossbeam::channel::{unbounded, TryRecvError};
use notify::{
    event::{DataChange, ModifyKind, RemoveKind, RenameMode},
    recommended_watcher, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
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
            let index_meta_re = Regex::new(INDEX_METADATA_FILE_PATTERN).unwrap();

            let (tx_filesystem, rx_filesystem) = unbounded();

            let mut watcher: RecommendedWatcher =
                recommended_watcher(tx_filesystem).expect("Watch error.");
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

                        if index_meta_re.is_match(path.to_str().unwrap_or("")) {
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
                                    // Load index metadata.
                                    info!(?path, "Load index metadata.");
                                    let index_metadata = match load_index_metadata(path).await {
                                        Ok(index_metadata) => index_metadata,
                                        Err(error) => {
                                            error!(?path, ?error, "Failed to load metadata.");
                                            continue;
                                        }
                                    };

                                    // Update index metadatas.
                                    info!(?index_name, "Insert index metadata.");
                                    let mut mut_metadatas = metadatas_task.write().await;
                                    mut_metadatas
                                        .insert(index_name.clone(), index_metadata.clone());

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
