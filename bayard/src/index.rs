pub mod analyzer;
pub mod metadata;
pub mod metastore;
pub mod shard;
pub mod shards;
pub mod token_filter;
pub mod tokenizer;

pub const DOC_ID_FIELD_NAME: &str = "_id";
pub const DOC_TIMESTAMP_FIELD_NAME: &str = "_timestamp";

#[cfg(not(target_os = "windows"))]
const INDEX_METADATA_FILE_PATTERN: &str = r"^.*/indices/([^/]+)/meta.json$";
#[cfg(target_os = "windows")]
const INDEX_METADATA_FILE_PATTERN: &str = r"^.*\indices\([^\]+)\meta.json$";

pub const INDICES_DIR: &str = "indices";
pub const INDEX_METADATA_FILE: &str = "meta.json";
pub const SHARDS_DIR: &str = "shards";
