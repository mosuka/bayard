use std::path::Path;

use regex::Regex;
use tantivy::{directory::MmapDirectory, schema::Schema, Index, IndexSettings};
use tokio::fs;

use crate::index::analyzer::{register, Analyzers};

use super::{NodeError, NodeErrorKind};

#[cfg(not(target_os = "windows"))]
const INDEX_NAME_PATTERN: &str = r"^.*/indices/([^/]+).*$";
#[cfg(target_os = "windows")]
const INDEX_NAME_PATTERN: &str = r"^.*\indices\([^\]+).*$";

pub async fn index_exists(index_data_dir: &Path) -> Result<bool, NodeError> {
    if !index_data_dir.exists() {
        return Ok(false);
    }

    let index_mmap_dir = MmapDirectory::open(index_data_dir)
        .map_err(|error| NodeErrorKind::IndexDirectoryCreationFailure.with_error(error))?;

    Index::exists(&index_mmap_dir)
        .map_err(|error| NodeErrorKind::IndexExistenceFailure.with_error(error))
}

pub async fn create_index(
    index_data_dir: &Path,
    schema: &Schema,
    index_settings: &IndexSettings,
    analyzers: &Analyzers,
) -> Result<Index, NodeError> {
    if index_exists(index_data_dir).await? {
        return Err(NodeErrorKind::IndexExists.with_error(anyhow::anyhow!(
            "Index {:?} already exists.",
            index_data_dir
        )));
    }

    fs::create_dir_all(&index_data_dir)
        .await
        .map_err(|error| NodeErrorKind::DirectoryCreationFailure.with_error(error))?;

    // Create directory.
    let index_mmap_dir = MmapDirectory::open(index_data_dir)
        .map_err(|error| NodeErrorKind::IndexDirectoryCreationFailure.with_error(error))?;

    // Create index.
    let index = Index::create(index_mmap_dir, schema.clone(), index_settings.clone())
        .map_err(|error| NodeErrorKind::IndexCreationFailure.with_error(error))?;

    // Register analyzers.
    register(index.tokenizers(), analyzers)
        .map_err(|error| NodeErrorKind::AnalyzerRegistrationFailure.with_error(error))?;

    Ok(index)
}

pub async fn open_index(index_data_dir: &Path, analyzers: &Analyzers) -> Result<Index, NodeError> {
    if !index_exists(index_data_dir).await? {
        return Err(NodeErrorKind::IndexNotFound.with_error(anyhow::anyhow!(
            "Index {:?} does not exist.",
            index_data_dir
        )));
    }

    let index_mmap_dir = MmapDirectory::open(index_data_dir)
        .map_err(|error| NodeErrorKind::DirectoryCreationFailure.with_error(error))?;

    let index = Index::open(index_mmap_dir)
        .map_err(|error| NodeErrorKind::IndexOpenFailure.with_error(error))?;

    // Register analyzers.
    register(index.tokenizers(), analyzers)
        .map_err(|error| NodeErrorKind::AnalyzerRegistrationFailure.with_error(error))?;

    Ok(index)
}

pub async fn delete_index(index_data_dir: &Path) -> Result<(), NodeError> {
    fs::remove_dir_all(index_data_dir)
        .await
        .map_err(|error| NodeErrorKind::IndexDeletionFailure.with_error(error))
}

pub fn extract_index_name(path: &Path) -> Result<String, NodeError> {
    let re = Regex::new(INDEX_NAME_PATTERN)
        .map_err(|error| NodeErrorKind::InvalidPath.with_error(error))?;

    // Get path string.
    let path_str = match path.to_str() {
        Some(path_str) => path_str,
        None => {
            return Err(NodeErrorKind::InvalidPath
                .with_error(anyhow::anyhow!("No index name part in path.")))
        }
    };

    let caps = re.captures(path_str).ok_or_else(|| {
        NodeErrorKind::InvalidPath.with_error(anyhow::anyhow!("Invalid Path: {}.", path_str))
    })?;
    let index_name = caps
        .get(1)
        .ok_or_else(|| {
            NodeErrorKind::InvalidPath.with_error(anyhow::anyhow!("Invalid Path: {}.", path_str))
        })?
        .as_str();

    Ok(index_name.to_string())
}
