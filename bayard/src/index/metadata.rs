use std::{
    cmp::Ordering,
    fmt,
    path::Path,
    sync::{Arc, RwLock},
};

use rand::{distributions::Alphanumeric, Rng};
use serde::{
    de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor},
    ser::{self, Serialize, SerializeStruct, Serializer},
};
use tantivy::{
    schema::{
        Cardinality, IndexRecordOption, NumericOptions, Schema, TextFieldIndexing, TextOptions,
    },
    IndexSettings,
};

use crate::{
    common::read_file,
    index::{analyzer::Analyzers, shard::Shard, shards::Shards},
};

use super::{DOC_ID_FIELD_NAME, DOC_TIMESTAMP_FIELD_NAME};

const SHARD_ID_LENGTH: usize = 8;

const DEFAULT_WRITER_THREADS: usize = 1;
const DEFAULT_WRITER_MEM_SIZE: usize = 100_000_000;
const DEFAULT_NUM_REPLICAS: usize = 1;
const DEFAULT_NUM_SHARDS: usize = 1;

fn generate_shard_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(SHARD_ID_LENGTH)
        .map(char::from)
        .collect()
}

#[derive(Debug, Clone, Copy)]
pub enum MetadataErrorKind {
    RwLockFailure,
    InvalidArgument,
    SchemaError,
    SerializationFailure,
    SaveFailure,
    LoadFailure,
    DeserializationFailure,
}

impl MetadataErrorKind {
    pub fn with_error<E>(self, source: E) -> MetadataError
    where
        anyhow::Error: From<E>,
    {
        MetadataError {
            kind: self,
            source: From::from(source),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("MetadataError(kind={kind:?}, source={source})")]
pub struct MetadataError {
    pub kind: MetadataErrorKind,
    #[source]
    source: anyhow::Error,
}

impl MetadataError {
    pub fn add_context<C>(self, ctx: C) -> Self
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        MetadataError {
            kind: self.kind,
            source: self.source.context(ctx),
        }
    }

    pub fn kind(&self) -> MetadataErrorKind {
        self.kind
    }
}

#[derive(Clone)]
pub struct Metadata {
    schema: Arc<RwLock<Schema>>,
    analyzers: Arc<RwLock<Analyzers>>,
    index_settings: Arc<RwLock<IndexSettings>>,
    writer_threads: Arc<RwLock<usize>>,
    writer_mem_size: Arc<RwLock<usize>>,
    num_replicas: Arc<RwLock<usize>>,
    num_shards: Arc<RwLock<usize>>,
    shards: Arc<RwLock<Shards>>,
}

impl Metadata {
    pub fn new(
        schema: Schema,
        analyzers: Analyzers,
        index_settings: IndexSettings,
        writer_threads: usize,
        writer_mem_size: usize,
        num_replicas: usize,
        num_shards: usize,
    ) -> Self {
        let mut shards = Shards::new();
        for _ in 0..num_shards {
            shards.push(Shard::new(generate_shard_id()));
        }

        Metadata {
            schema: Arc::new(RwLock::new(schema)),
            analyzers: Arc::new(RwLock::new(analyzers)),
            index_settings: Arc::new(RwLock::new(index_settings)),
            writer_threads: Arc::new(RwLock::new(writer_threads)),
            writer_mem_size: Arc::new(RwLock::new(writer_mem_size)),
            num_replicas: Arc::new(RwLock::new(num_replicas)),
            num_shards: Arc::new(RwLock::new(num_shards)),
            shards: Arc::new(RwLock::new(shards)),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_shards(
        schema: Schema,
        analyzers: Analyzers,
        index_settings: IndexSettings,
        writer_threads: usize,
        writer_mem_size: usize,
        num_replicas: usize,
        num_shards: usize,
        shards: Shards,
    ) -> Result<Self, MetadataError> {
        let mut metadata = Metadata {
            schema: Arc::new(RwLock::new(schema)),
            analyzers: Arc::new(RwLock::new(analyzers)),
            index_settings: Arc::new(RwLock::new(index_settings)),
            writer_threads: Arc::new(RwLock::new(writer_threads)),
            writer_mem_size: Arc::new(RwLock::new(writer_mem_size)),
            num_replicas: Arc::new(RwLock::new(num_replicas)),
            num_shards: Arc::new(RwLock::new(num_shards)),
            shards: Arc::new(RwLock::new(shards.clone())),
        };

        // If the `shards` length and the `num_shards` are different,
        // the `shards` value takes precedence over the `num_shards`.
        if num_shards != shards.len() {
            metadata.set_shards(shards)?;
        }

        Ok(metadata)
    }

    pub fn schema(&self) -> Result<Schema, MetadataError> {
        Ok(self
            .schema
            .read()
            .map_err(|error| {
                MetadataErrorKind::RwLockFailure.with_error(anyhow::anyhow!(error.to_string()))
            })?
            .clone())
    }

    pub fn set_schema(&mut self, new_schema: Schema) -> Result<Schema, MetadataError> {
        let mut schema = self.schema.write().map_err(|error| {
            MetadataErrorKind::RwLockFailure.with_error(anyhow::anyhow!(error.to_string()))
        })?;

        *schema = new_schema;

        Ok(schema.clone())
    }

    pub fn analyzers(&self) -> Result<Analyzers, MetadataError> {
        Ok(self
            .analyzers
            .read()
            .map_err(|error| {
                MetadataErrorKind::RwLockFailure.with_error(anyhow::anyhow!(error.to_string()))
            })?
            .clone())
    }

    pub fn set_analyzers(&mut self, new_analyzers: Analyzers) -> Result<Analyzers, MetadataError> {
        let mut analyzers = self.analyzers.write().map_err(|error| {
            MetadataErrorKind::RwLockFailure.with_error(anyhow::anyhow!(error.to_string()))
        })?;

        *analyzers = new_analyzers;

        Ok(analyzers.clone())
    }

    pub fn index_settings(&self) -> Result<IndexSettings, MetadataError> {
        Ok(self
            .index_settings
            .read()
            .map_err(|error| {
                MetadataErrorKind::RwLockFailure.with_error(anyhow::anyhow!(error.to_string()))
            })?
            .clone())
    }

    pub fn set_index_settings(
        &mut self,
        new_index_settings: IndexSettings,
    ) -> Result<IndexSettings, MetadataError> {
        let mut index_settings = self.index_settings.write().map_err(|error| {
            MetadataErrorKind::RwLockFailure.with_error(anyhow::anyhow!(error.to_string()))
        })?;

        *index_settings = new_index_settings;

        Ok(index_settings.clone())
    }

    pub fn writer_threads(&self) -> Result<usize, MetadataError> {
        Ok(*self.writer_threads.read().map_err(|error| {
            MetadataErrorKind::RwLockFailure.with_error(anyhow::anyhow!(error.to_string()))
        })?)
    }

    pub fn set_writer_threads(
        &mut self,
        new_writer_threads: usize,
    ) -> Result<usize, MetadataError> {
        if new_writer_threads == 0 {
            return Err(MetadataErrorKind::InvalidArgument
                .with_error(anyhow::anyhow!("Writer threads must be greater than 0")));
        }

        let mut writer_threads = self.writer_threads.write().map_err(|error| {
            MetadataErrorKind::RwLockFailure.with_error(anyhow::anyhow!(error.to_string()))
        })?;

        *writer_threads = new_writer_threads;

        Ok(*writer_threads)
    }

    pub fn writer_mem_size(&self) -> Result<usize, MetadataError> {
        Ok(*self.writer_mem_size.read().map_err(|error| {
            MetadataErrorKind::RwLockFailure.with_error(anyhow::anyhow!(error.to_string()))
        })?)
    }

    pub fn set_writer_mem_size(
        &mut self,
        new_writer_mem_size: usize,
    ) -> Result<usize, MetadataError> {
        if new_writer_mem_size == 0 {
            return Err(MetadataErrorKind::InvalidArgument
                .with_error(anyhow::anyhow!("Writer memory size must be greater than 0")));
        }

        let mut writer_mem_size = self.writer_mem_size.write().map_err(|error| {
            MetadataErrorKind::RwLockFailure.with_error(anyhow::anyhow!(error.to_string()))
        })?;

        *writer_mem_size = new_writer_mem_size;

        Ok(*writer_mem_size)
    }

    pub fn num_replicas(&self) -> Result<usize, MetadataError> {
        Ok(*self.num_replicas.read().map_err(|error| {
            MetadataErrorKind::RwLockFailure.with_error(anyhow::anyhow!(error.to_string()))
        })?)
    }

    pub fn set_num_replicas(&mut self, num_replicas: usize) -> Result<usize, MetadataError> {
        if num_replicas == 0 {
            return Err(MetadataErrorKind::InvalidArgument
                .with_error(anyhow::anyhow!("Number of replicas must be greater than 0")));
        }

        let mut current_num_replicas = self.num_replicas.write().map_err(|error| {
            MetadataErrorKind::RwLockFailure.with_error(anyhow::anyhow!(error.to_string()))
        })?;

        *current_num_replicas = num_replicas;

        Ok(*current_num_replicas)
    }

    pub fn num_shards(&self) -> Result<usize, MetadataError> {
        Ok(*self.num_shards.read().map_err(|error| {
            MetadataErrorKind::RwLockFailure.with_error(anyhow::anyhow!(error.to_string()))
        })?)
    }

    pub fn set_num_shards(&mut self, num_shards: usize) -> Result<usize, MetadataError> {
        if num_shards == 0 {
            return Err(MetadataErrorKind::InvalidArgument
                .with_error(anyhow::anyhow!("Number of shards must be greater than 0")));
        }

        let mut current_num_shards = self.num_shards.write().map_err(|error| {
            MetadataErrorKind::RwLockFailure.with_error(anyhow::anyhow!(error.to_string()))
        })?;
        let mut current_shards = self.shards.write().map_err(|error| {
            MetadataErrorKind::RwLockFailure.with_error(anyhow::anyhow!(error.to_string()))
        })?;

        *current_num_shards = num_shards;
        match current_shards.len().cmp(&num_shards) {
            Ordering::Equal => {
                // Nothing to do
            }
            Ordering::Greater => {
                let cnt = current_shards.len() - num_shards;
                for _ in 0..cnt {
                    current_shards.pop();
                }
            }
            Ordering::Less => {
                let cnt = num_shards - current_shards.len();
                for _ in 0..cnt {
                    current_shards.push(Shard::new(generate_shard_id()));
                }
            }
        }

        Ok(*current_num_shards)
    }

    pub fn shards(&self) -> Result<Shards, MetadataError> {
        Ok(self
            .shards
            .read()
            .map_err(|error| {
                MetadataErrorKind::RwLockFailure.with_error(anyhow::anyhow!(error.to_string()))
            })?
            .clone())
    }

    pub fn set_shards(&mut self, shards: Shards) -> Result<Shards, MetadataError> {
        let mut current_shards = self.shards.write().map_err(|error| {
            MetadataErrorKind::RwLockFailure.with_error(anyhow::anyhow!(error.to_string()))
        })?;
        let mut current_num_shards = self.num_shards.write().map_err(|error| {
            MetadataErrorKind::RwLockFailure.with_error(anyhow::anyhow!(error.to_string()))
        })?;

        *current_num_shards = shards.len();
        *current_shards = shards;

        Ok(current_shards.clone())
    }
}

impl Serialize for Metadata {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Meta", 7)?;
        state.serialize_field(
            "schema",
            &self
                .schema()
                .map_err(|error| ser::Error::custom(error.to_string()))?,
        )?;
        state.serialize_field(
            "index_settings",
            &self
                .index_settings()
                .map_err(|error| ser::Error::custom(error.to_string()))?,
        )?;
        state.serialize_field(
            "analyzers",
            &self
                .analyzers()
                .map_err(|error| ser::Error::custom(error.to_string()))?,
        )?;
        state.serialize_field(
            "writer_threads",
            &self
                .writer_threads()
                .map_err(|error| ser::Error::custom(error.to_string()))?,
        )?;
        state.serialize_field(
            "writer_mem_size",
            &self
                .writer_mem_size()
                .map_err(|error| ser::Error::custom(error.to_string()))?,
        )?;

        state.serialize_field(
            "num_replicas",
            &self
                .num_replicas()
                .map_err(|error| ser::Error::custom(error.to_string()))?,
        )?;

        state.serialize_field(
            "num_shards",
            &self
                .num_shards()
                .map_err(|error| ser::Error::custom(error.to_string()))?,
        )?;

        state.serialize_field(
            "shards",
            &self
                .shards()
                .map_err(|error| ser::Error::custom(error.to_string()))?,
        )?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Metadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "schema",
            "index_settings",
            "analyzers",
            "writer_threads",
            "writer_mem_size",
            "num_replicas",
            "num_shards",
            "shards",
        ];

        enum Field {
            Schema,
            IndexSettings,
            Analyzers,
            WriterThread,
            WriterMemSize,
            NumReplicas,
            NumShards,
            Shards,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`schema`, `analyzers`, `index_settings`, `writer_threads`, `writer_mem_size`, `num_replicas`, `num_shards` or `shards`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "schema" => Ok(Field::Schema),
                            "analyzers" => Ok(Field::Analyzers),
                            "index_settings" => Ok(Field::IndexSettings),
                            "writer_threads" => Ok(Field::WriterThread),
                            "writer_mem_size" => Ok(Field::WriterMemSize),
                            "num_replicas" => Ok(Field::NumReplicas),
                            "num_shards" => Ok(Field::NumShards),
                            "shards" => Ok(Field::Shards),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct IndexMetadataVisitor;

        impl<'de> Visitor<'de> for IndexMetadataVisitor {
            type Value = Metadata;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct IndexMetadata")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Metadata, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let schema = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let analyzers = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let index_settings = seq.next_element()?.unwrap_or_default();
                let writer_threads = seq.next_element()?.unwrap_or(DEFAULT_WRITER_THREADS);
                let writer_mem_size = seq.next_element()?.unwrap_or(DEFAULT_WRITER_MEM_SIZE);
                let num_replicas = seq.next_element()?.unwrap_or(DEFAULT_NUM_REPLICAS);
                let num_shards = seq.next_element()?.unwrap_or(DEFAULT_NUM_SHARDS);
                let shards = seq.next_element()?;

                if let Some(shard_ids) = shards {
                    Ok(Metadata::new_with_shards(
                        schema,
                        analyzers,
                        index_settings,
                        writer_threads,
                        writer_mem_size,
                        num_replicas,
                        num_shards,
                        shard_ids,
                    )
                    .map_err(|error| de::Error::custom(error.to_string()))?)
                } else {
                    Ok(Metadata::new(
                        schema,
                        analyzers,
                        index_settings,
                        writer_threads,
                        writer_mem_size,
                        num_replicas,
                        num_shards,
                    ))
                }
            }

            fn visit_map<V>(self, mut map: V) -> Result<Metadata, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut schema = None;
                let mut analyzers = None;
                let mut index_settings = None;
                let mut writer_threads = None;
                let mut writer_mem_size = None;
                let mut num_replicas = None;
                let mut num_shards = None;
                let mut shards = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Schema => {
                            if schema.is_some() {
                                return Err(de::Error::duplicate_field("schema"));
                            }
                            schema = Some(map.next_value()?);
                        }
                        Field::Analyzers => {
                            if analyzers.is_some() {
                                return Err(de::Error::duplicate_field("analyzers"));
                            }
                            analyzers = Some(map.next_value()?);
                        }
                        Field::IndexSettings => {
                            if index_settings.is_some() {
                                return Err(de::Error::duplicate_field("index_settings"));
                            }
                            index_settings = Some(map.next_value()?);
                        }
                        Field::WriterThread => {
                            if writer_threads.is_some() {
                                return Err(de::Error::duplicate_field("writer_threads"));
                            }
                            writer_threads = Some(map.next_value()?);
                        }
                        Field::WriterMemSize => {
                            if writer_mem_size.is_some() {
                                return Err(de::Error::duplicate_field("writer_mem_size"));
                            }
                            writer_mem_size = Some(map.next_value()?);
                        }
                        Field::NumReplicas => {
                            if num_replicas.is_some() {
                                return Err(de::Error::duplicate_field("num_replicas"));
                            }
                            num_replicas = Some(map.next_value()?);
                        }
                        Field::NumShards => {
                            if num_shards.is_some() {
                                return Err(de::Error::duplicate_field("num_shards"));
                            }
                            num_shards = Some(map.next_value()?);
                        }
                        Field::Shards => {
                            if shards.is_some() {
                                return Err(de::Error::duplicate_field("shards"));
                            }
                            shards = Some(map.next_value()?);
                        }
                    }
                }
                let schema = schema.ok_or_else(|| de::Error::missing_field("schema"))?;
                let analyzers = analyzers.ok_or_else(|| de::Error::missing_field("analyzers"))?;
                let index_settings = index_settings.unwrap_or_default();
                let writer_threads = writer_threads.unwrap_or(DEFAULT_WRITER_THREADS);
                let writer_mem_size = writer_mem_size.unwrap_or(DEFAULT_WRITER_MEM_SIZE);
                let num_replicas = num_replicas.unwrap_or(DEFAULT_NUM_REPLICAS);
                let num_shards = num_shards.unwrap_or(DEFAULT_NUM_SHARDS);

                if let Some(shards) = shards {
                    Ok(Metadata::new_with_shards(
                        schema,
                        analyzers,
                        index_settings,
                        writer_threads,
                        writer_mem_size,
                        num_replicas,
                        num_shards,
                        shards,
                    )
                    .map_err(|error| de::Error::custom(error.to_string()))?)
                } else {
                    Ok(Metadata::new(
                        schema,
                        analyzers,
                        index_settings,
                        writer_threads,
                        writer_mem_size,
                        num_replicas,
                        num_shards,
                    ))
                }
            }
        }

        deserializer.deserialize_struct("Meta", FIELDS, IndexMetadataVisitor)
    }
}

pub async fn save_index_metadata(path: &Path, mut meta: Metadata) -> Result<(), MetadataError> {
    let mut schema_builder = Schema::builder();

    // Add fields other than reserved fields.
    for (_field, field_entry) in meta
        .schema()
        .map_err(|error| MetadataErrorKind::SchemaError.with_error(error))?
        .fields()
    {
        if field_entry.name() != DOC_ID_FIELD_NAME && field_entry.name() != DOC_TIMESTAMP_FIELD_NAME
        {
            schema_builder.add_field(field_entry.clone());
        }
    }

    meta.set_schema(schema_builder.build())
        .map_err(|error| MetadataErrorKind::SchemaError.with_error(error))?;

    let meta_bytes = serde_json::to_vec(&meta)
        .map_err(|error| MetadataErrorKind::SerializationFailure.with_error(error))?;

    crate::common::write_file(path, meta_bytes.as_slice())
        .await
        .map_err(|error| MetadataErrorKind::SaveFailure.with_error(error))
}

pub async fn load_index_metadata(path: &Path) -> Result<Metadata, MetadataError> {
    let meta_bytes = read_file(path)
        .await
        .map_err(|error| MetadataErrorKind::LoadFailure.with_error(error))?;

    let mut meta = serde_json::from_slice::<Metadata>(&meta_bytes)
        .map_err(|error| MetadataErrorKind::DeserializationFailure.with_error(error))?;

    let mut schema_builder = Schema::builder();

    // Add document ID field.
    let id_options = TextOptions::default().set_stored().set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer("raw")
            .set_index_option(IndexRecordOption::Basic),
    );
    schema_builder.add_text_field(DOC_ID_FIELD_NAME, id_options);

    // Add document timestamp field.
    let timestamp_options = NumericOptions::default()
        .set_stored()
        .set_indexed()
        .set_fieldnorm()
        .set_fast(Cardinality::SingleValue);
    schema_builder.add_date_field(DOC_TIMESTAMP_FIELD_NAME, timestamp_options);

    // Add fields other than reserved fields.
    for (_field, field_entry) in meta
        .schema()
        .map_err(|error| MetadataErrorKind::SchemaError.with_error(error))?
        .fields()
    {
        if field_entry.name() != DOC_ID_FIELD_NAME && field_entry.name() != DOC_TIMESTAMP_FIELD_NAME
        {
            schema_builder.add_field(field_entry.clone());
        }
    }

    meta.set_schema(schema_builder.build())
        .map_err(|error| MetadataErrorKind::SchemaError.with_error(error))?;

    Ok(meta)
}

#[cfg(test)]
mod tests {
    use tantivy::{schema::Schema, IndexSettings};

    use crate::index::{
        analyzer::Analyzers,
        metadata::{
            generate_shard_id, Metadata, DEFAULT_NUM_REPLICAS, DEFAULT_NUM_SHARDS,
            DEFAULT_WRITER_MEM_SIZE, DEFAULT_WRITER_THREADS,
        },
        shard::Shard,
        shards::Shards,
    };

    #[test]
    fn test_generate_shard_id() {
        let shard_id = generate_shard_id();

        assert_eq!(shard_id.len(), 8);
    }

    #[test]
    fn test_index_metadata_from_slice() {
        let meta_json_str = r#"
        {
            "analyzers": {
                "default": {
                    "tokenizer": {
                        "name": "simple"
                    },
                    "filters": [
                        {
                            "name": "remove_long",
                            "args": {
                                "length_limit": 40
                            }
                        },
                        {
                            "name": "ascii_folding"
                        },
                        {
                            "name": "lower_case"
                        }
                    ]
                }
            },
            "schema": [
                {
                    "name": "name",
                    "type": "text",
                    "options": {
                        "indexing": {
                            "record": "position",
                            "fieldnorms": false,
                            "tokenizer": "default"
                        },
                        "stored": true
                    }
                }
            ],
            "writer_threads": 1,
            "writer_mem_size": 500000000,
            "index_settings": {
                "sort_by_field": null,
                "docstore_compression": "none"
            },
            "num_replicas": 2,
            "num_shards": 2
        }
        "#;
        let meta_json_bytes = meta_json_str.as_bytes();

        let meta = serde_json::from_slice::<Metadata>(meta_json_bytes).unwrap();
        assert_eq!(meta.writer_threads().unwrap(), 1);
        assert_eq!(meta.writer_mem_size().unwrap(), 500000000);
        assert_eq!(meta.num_replicas().unwrap(), 2);
        assert_eq!(meta.num_shards().unwrap(), 2);
        assert_eq!(meta.shards().unwrap().len(), 2);

        let meta_json_str = r#"
        {
            "analyzers": {
                "default": {
                    "tokenizer": {
                        "name": "simple"
                    },
                    "filters": [
                        {
                            "name": "remove_long",
                            "args": {
                                "length_limit": 40
                            }
                        },
                        {
                            "name": "ascii_folding"
                        },
                        {
                            "name": "lower_case"
                        }
                    ]
                }
            },
            "schema": [
                {
                    "name": "name",
                    "type": "text",
                    "options": {
                        "indexing": {
                            "record": "position",
                            "fieldnorms": false,
                            "tokenizer": "default"
                        },
                        "stored": true
                    }
                }
            ],
            "writer_threads": 1,
            "writer_mem_size": 500000000,
            "index_settings": {
                "sort_by_field": null,
                "docstore_compression": "none"
            },
            "num_replicas": 2,
            "num_shards": 3,
            "shards": {
                "shard_list": [
                    {
                        "id": "shard-1"
                    },
                    {
                        "id": "shard-2"
                    },
                    {
                        "id": "shard-3"
                    }
                ]
            }
        }
        "#;
        let meta_json_bytes = meta_json_str.as_bytes();

        let meta = serde_json::from_slice::<Metadata>(meta_json_bytes).unwrap();
        assert_eq!(meta.writer_threads().unwrap(), 1);
        assert_eq!(meta.writer_mem_size().unwrap(), 500000000);
        assert_eq!(meta.num_replicas().unwrap(), 2);
        assert_eq!(meta.num_shards().unwrap(), 3);
        let shards = meta.shards().unwrap();
        assert_eq!(shards.len(), 3);
        assert!(shards.contains(&"shard-1".to_string()));
        assert!(shards.contains(&"shard-2".to_string()));
        assert!(shards.contains(&"shard-3".to_string()));
    }

    #[test]
    fn test_index_metadata_set_schema() {
        let meta_json_str = r#"
        {
            "analyzers": {
                "default": {
                    "tokenizer": {
                        "name": "simple"
                    },
                    "filters": [
                        {
                            "name": "remove_long",
                            "args": {
                                "length_limit": 40
                            }
                        },
                        {
                            "name": "ascii_folding"
                        },
                        {
                            "name": "lower_case"
                        }
                    ]
                }
            },
            "schema": [
                {
                    "name": "name",
                    "type": "text",
                    "options": {
                        "indexing": {
                            "record": "position",
                            "fieldnorms": false,
                            "tokenizer": "default"
                        },
                        "stored": true
                    }
                }
            ],
            "writer_threads": 1,
            "writer_mem_size": 500000000,
            "index_settings": {
                "sort_by_field": null,
                "docstore_compression": "none",
                "docstore_blocksize": 16384
            },
            "num_replicas": 2,
            "num_shards": 2
        }
        "#;
        let meta_json_bytes = meta_json_str.as_bytes();

        let mut meta = serde_json::from_slice::<Metadata>(meta_json_bytes).unwrap();

        let schema = meta.schema().unwrap();
        assert_eq!(schema.num_fields(), 1);

        let new_schema_json_str = r#"
        [
            {
                "name": "name",
                "type": "text",
                "options": {
                    "indexing": {
                        "record": "position",
                        "fieldnorms": false,
                        "tokenizer": "default"
                    },
                    "stored": true
                }
            },
            {
                "name": "description",
                "type": "text",
                "options": {
                    "indexing": {
                        "record": "position",
                        "fieldnorms": false,
                        "tokenizer": "default"
                    },
                    "stored": true
                }
            }
        ]
        "#;
        let new_schema_json_bytes = new_schema_json_str.as_bytes();
        let new_schema = serde_json::from_slice::<Schema>(new_schema_json_bytes).unwrap();
        let new_schema = meta.set_schema(new_schema).unwrap();
        assert_eq!(new_schema.num_fields(), 2);
    }

    #[test]
    fn test_index_metadata_set_index_settings() {
        let meta_json_str = r#"
        {
            "analyzers": {
                "default": {
                    "tokenizer": {
                        "name": "simple"
                    },
                    "filters": [
                        {
                            "name": "remove_long",
                            "args": {
                                "length_limit": 40
                            }
                        },
                        {
                            "name": "ascii_folding"
                        },
                        {
                            "name": "lower_case"
                        }
                    ]
                }
            },
            "schema": [
                {
                    "name": "name",
                    "type": "text",
                    "options": {
                        "indexing": {
                            "record": "position",
                            "fieldnorms": false,
                            "tokenizer": "default"
                        },
                        "stored": true
                    }
                }
            ],
            "writer_threads": 1,
            "writer_mem_size": 500000000,
            "index_settings": {
                "sort_by_field": null,
                "docstore_compression": "none",
                "docstore_blocksize": 16384
            },
            "num_replicas": 2,
            "num_shards": 2
        }
        "#;
        let meta_json_bytes = meta_json_str.as_bytes();

        let mut meta = serde_json::from_slice::<Metadata>(meta_json_bytes).unwrap();

        let index_settings = meta.index_settings().unwrap();
        assert_eq!(index_settings.docstore_blocksize, 16384);

        let new_index_settings_json_str = r#"
        {
            "sort_by_field": null,
            "docstore_compression": "none",
            "docstore_blocksize": 10000
        }
        "#;
        let new_index_settings_json_bytes = new_index_settings_json_str.as_bytes();
        let new_index_settings =
            serde_json::from_slice::<IndexSettings>(new_index_settings_json_bytes).unwrap();
        let new_index_settings = meta.set_index_settings(new_index_settings).unwrap();
        assert_eq!(new_index_settings.docstore_blocksize, 10000);
    }

    #[test]
    fn test_index_metadata_set_analyzers() {
        let meta_json_str = r#"
        {
            "analyzers": {
                "default": {
                    "tokenizer": {
                        "name": "simple"
                    },
                    "filters": [
                        {
                            "name": "remove_long",
                            "args": {
                                "length_limit": 40
                            }
                        },
                        {
                            "name": "ascii_folding"
                        },
                        {
                            "name": "lower_case"
                        }
                    ]
                }
            },
            "schema": [
                {
                    "name": "name",
                    "type": "text",
                    "options": {
                        "indexing": {
                            "record": "position",
                            "fieldnorms": false,
                            "tokenizer": "default"
                        },
                        "stored": true
                    }
                }
            ],
            "writer_threads": 1,
            "writer_mem_size": 500000000,
            "index_settings": {
                "sort_by_field": null,
                "docstore_compression": "none"
            },
            "num_replicas": 2,
            "num_shards": 2
        }
        "#;
        let meta_json_bytes = meta_json_str.as_bytes();

        let mut meta = serde_json::from_slice::<Metadata>(meta_json_bytes).unwrap();

        let analyzers = meta.analyzers().unwrap();
        assert_eq!(analyzers.contains_key("default"), true);

        let new_analyzers_json_str = r#"
        {
            "default": {
                "tokenizer": {
                    "name": "whitespace"
                },
                "filters": [
                    {
                        "name": "remove_long",
                        "args": {
                            "length_limit": 40
                        }
                    },
                    {
                        "name": "ascii_folding"
                    },
                    {
                        "name": "lower_case"
                    }
                ]
            }
        }
        "#;
        let new_analyzers_json_bytes = new_analyzers_json_str.as_bytes();
        let new_analyzers = serde_json::from_slice::<Analyzers>(new_analyzers_json_bytes).unwrap();
        let new_analyzers = meta.set_analyzers(new_analyzers).unwrap();
        assert_eq!(new_analyzers.contains_key("default"), true);
    }

    #[test]
    fn test_index_metadata_set_writer_threads() {
        let meta_json_str = r#"
        {
            "analyzers": {
                "default": {
                    "tokenizer": {
                        "name": "simple"
                    },
                    "filters": [
                        {
                            "name": "remove_long",
                            "args": {
                                "length_limit": 40
                            }
                        },
                        {
                            "name": "ascii_folding"
                        },
                        {
                            "name": "lower_case"
                        }
                    ]
                }
            },
            "schema": [
                {
                    "name": "name",
                    "type": "text",
                    "options": {
                        "indexing": {
                            "record": "position",
                            "fieldnorms": false,
                            "tokenizer": "default"
                        },
                        "stored": true
                    }
                }
            ],
            "writer_threads": 1,
            "writer_mem_size": 500000000,
            "index_settings": {
                "sort_by_field": null,
                "docstore_compression": "none"
            },
            "num_replicas": 2,
            "num_shards": 2
        }
        "#;
        let meta_json_bytes = meta_json_str.as_bytes();

        let mut meta = serde_json::from_slice::<Metadata>(meta_json_bytes).unwrap();
        assert_eq!(meta.writer_threads().unwrap(), 1);

        let new_writer_threads = meta.set_writer_threads(3).unwrap();
        assert_eq!(new_writer_threads, 3);
        assert_eq!(meta.writer_threads().unwrap(), 3);
    }

    #[test]
    fn test_index_metadata_set_writer_mem_size() {
        let meta_json_str = r#"
        {
            "analyzers": {
                "default": {
                    "tokenizer": {
                        "name": "simple"
                    },
                    "filters": [
                        {
                            "name": "remove_long",
                            "args": {
                                "length_limit": 40
                            }
                        },
                        {
                            "name": "ascii_folding"
                        },
                        {
                            "name": "lower_case"
                        }
                    ]
                }
            },
            "schema": [
                {
                    "name": "name",
                    "type": "text",
                    "options": {
                        "indexing": {
                            "record": "position",
                            "fieldnorms": false,
                            "tokenizer": "default"
                        },
                        "stored": true
                    }
                }
            ],
            "writer_threads": 1,
            "writer_mem_size": 500000000,
            "index_settings": {
                "sort_by_field": null,
                "docstore_compression": "none"
            },
            "num_replicas": 2,
            "num_shards": 2
        }
        "#;
        let meta_json_bytes = meta_json_str.as_bytes();

        let mut meta = serde_json::from_slice::<Metadata>(meta_json_bytes).unwrap();
        assert_eq!(meta.writer_mem_size().unwrap(), 500000000);

        let new_writer_mem_size = meta.set_writer_mem_size(100000000).unwrap();
        assert_eq!(new_writer_mem_size, 100000000);
        assert_eq!(meta.writer_mem_size().unwrap(), 100000000);
    }

    #[test]
    fn test_index_metadata_set_replicas() {
        let meta_json_str = r#"
        {
            "analyzers": {
                "default": {
                    "tokenizer": {
                        "name": "simple"
                    },
                    "filters": [
                        {
                            "name": "remove_long",
                            "args": {
                                "length_limit": 40
                            }
                        },
                        {
                            "name": "ascii_folding"
                        },
                        {
                            "name": "lower_case"
                        }
                    ]
                }
            },
            "schema": [
                {
                    "name": "name",
                    "type": "text",
                    "options": {
                        "indexing": {
                            "record": "position",
                            "fieldnorms": false,
                            "tokenizer": "default"
                        },
                        "stored": true
                    }
                }
            ],
            "writer_threads": 1,
            "writer_mem_size": 500000000,
            "index_settings": {
                "sort_by_field": null,
                "docstore_compression": "none"
            },
            "num_replicas": 2,
            "num_shards": 2
        }
        "#;
        let meta_json_bytes = meta_json_str.as_bytes();

        let mut meta = serde_json::from_slice::<Metadata>(meta_json_bytes).unwrap();
        assert_eq!(meta.num_replicas().unwrap(), 2);

        let new_replicas = meta.set_num_replicas(1).unwrap();
        assert_eq!(new_replicas, 1);
        assert_eq!(meta.num_replicas().unwrap(), 1);
    }

    #[test]
    fn test_index_metadata_set_shards() {
        let meta_json_str = r#"
        {
            "analyzers": {
                "default": {
                    "tokenizer": {
                        "name": "simple"
                    },
                    "filters": [
                        {
                            "name": "remove_long",
                            "args": {
                                "length_limit": 40
                            }
                        },
                        {
                            "name": "ascii_folding"
                        },
                        {
                            "name": "lower_case"
                        }
                    ]
                }
            },
            "schema": [
                {
                    "name": "name",
                    "type": "text",
                    "options": {
                        "indexing": {
                            "record": "position",
                            "fieldnorms": false,
                            "tokenizer": "default"
                        },
                        "stored": true
                    }
                }
            ],
            "writer_threads": 1,
            "writer_mem_size": 500000000,
            "index_settings": {
                "sort_by_field": null,
                "docstore_compression": "none"
            },
            "num_replicas": 2,
            "num_shards": 2,
            "shards": {
                "shard_list": [
                    {
                        "id": "shard-1"
                    },
                    {
                        "id": "shard-2"
                    }
                ]
            }
        }
        "#;
        let meta_json_bytes = meta_json_str.as_bytes();

        let mut meta = serde_json::from_slice::<Metadata>(meta_json_bytes).unwrap();
        assert_eq!(meta.num_shards().unwrap(), 2);

        let new_shards = meta.set_num_shards(1).unwrap();
        assert_eq!(new_shards, 1);
        assert_eq!(meta.num_shards().unwrap(), 1);
        let shards = meta.shards().unwrap();
        assert_eq!(shards.len(), 1);
        assert!(shards.contains(&"shard-1".to_string()));

        let new_shards = meta
            .set_shards(Shards::init(vec![
                Shard::new("shard-1".to_string()),
                Shard::new("shard-2".to_string()),
                Shard::new("shard-3".to_string()),
            ]))
            .unwrap();
        assert_eq!(new_shards.len(), 3);
        assert!(new_shards.contains(&"shard-1".to_string()));
        assert!(new_shards.contains(&"shard-2".to_string()));
        assert!(new_shards.contains(&"shard-3".to_string()));

        assert_eq!(meta.num_shards().unwrap(), 3);
        let shards = meta.shards().unwrap();
        assert_eq!(shards.len(), 3);
        assert!(shards.contains(&"shard-1".to_string()));
        assert!(shards.contains(&"shard-2".to_string()));
        assert!(shards.contains(&"shard-3".to_string()));
    }

    #[test]
    fn test_index_metadata_defaults() {
        let meta_json_str = r#"
        {
            "schema": [
                {
                    "name": "name",
                    "type": "text",
                    "options": {
                        "indexing": {
                            "record": "position",
                            "fieldnorms": false,
                            "tokenizer": "default"
                        },
                        "stored": true
                    }
                }
            ],
            "analyzers": {
                "default": {
                    "tokenizer": {
                        "name": "simple"
                    },
                    "filters": [
                        {
                            "name": "remove_long",
                            "args": {
                                "length_limit": 40
                            }
                        },
                        {
                            "name": "ascii_folding"
                        },
                        {
                            "name": "lower_case"
                        }
                    ]
                }
            }
        }
        "#;
        let meta_json_bytes = meta_json_str.as_bytes();

        let meta = serde_json::from_slice::<Metadata>(meta_json_bytes).unwrap();
        assert_eq!(meta.writer_threads().unwrap(), DEFAULT_WRITER_THREADS);
        assert_eq!(meta.writer_mem_size().unwrap(), DEFAULT_WRITER_MEM_SIZE);
        assert_eq!(meta.num_replicas().unwrap(), DEFAULT_NUM_REPLICAS);
        assert_eq!(meta.num_shards().unwrap(), DEFAULT_NUM_SHARDS);
        assert_eq!(meta.shards().unwrap().len(), DEFAULT_NUM_SHARDS);
    }
}
