use std::{collections::HashMap, fmt, path::Path, str::FromStr};

use serde::{
    de::{self, Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};
use serde_json::Value;
use tantivy::tokenizer::{TextAnalyzer, TokenizerManager};
use tokio::fs;

use super::{
    token_filter::{
        alpha_num_only::create_alpha_num_only_filter, ascii_folding::create_ascii_folding_filter,
        lower_case::create_lower_case_filter, remove_long::create_remove_long_filter,
        stemming::create_stemming_filter, stop_word::create_stop_word_filter, FilterKind,
    },
    tokenizer::{
        lindera::create_lindera_analyzer, ngram::create_ngram_analyzer, raw::create_raw_analyzer,
        simple::create_simple_analyzer, whitespace::create_whitespace_analyzer, TokenizerKind,
    },
};

#[derive(Debug, Clone, Copy)]
pub enum AnalyzerErrorKind {
    ParseFailure,
    InitializationFailure,
    InvalidTokenizerKind,
}

impl AnalyzerErrorKind {
    pub fn with_error<E>(self, source: E) -> AnalyzerError
    where
        anyhow::Error: From<E>,
    {
        AnalyzerError {
            kind: self,
            source: From::from(source),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("AnalyzerError(kind={kind:?}, source={source})")]
pub struct AnalyzerError {
    pub kind: AnalyzerErrorKind,
    #[source]
    source: anyhow::Error,
}

impl AnalyzerError {
    pub fn add_context<C>(self, ctx: C) -> Self
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        AnalyzerError {
            kind: self.kind,
            source: self.source.context(ctx),
        }
    }

    pub fn kind(&self) -> AnalyzerErrorKind {
        self.kind
    }
}

#[derive(Clone)]
pub struct Analyzers {
    source: Value,
    analyzers: HashMap<String, TextAnalyzer>,
}

impl Analyzers {
    pub async fn new(path: &Path) -> Result<Self, AnalyzerError> {
        let analyzers_bytes = fs::read(&path)
            .await
            .map_err(|error| AnalyzerErrorKind::InitializationFailure.with_error(error))?;
        serde_json::from_slice::<Analyzers>(&analyzers_bytes)
            .map_err(|error| AnalyzerErrorKind::InitializationFailure.with_error(error))
    }

    pub fn get(&self, name: &str) -> Option<&TextAnalyzer> {
        self.analyzers.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &TextAnalyzer)> {
        self.analyzers.iter()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.analyzers.contains_key(key)
    }
}

impl Serialize for Analyzers {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.source.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Analyzers {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = Value::deserialize(deserializer)?;
        let text_analyzers = build_analyzers(&inner).map_err(de::Error::custom)?;

        Ok(text_analyzers)
    }
}

fn build_analyzers(source: &Value) -> Result<Analyzers, AnalyzerError> {
    let mut analyzers = HashMap::new();

    let inner_map = source.as_object().ok_or_else(|| {
        AnalyzerErrorKind::ParseFailure.with_error(anyhow::anyhow!("src must be a map"))
    })?;

    for (analyzer_name, analyzer_setting) in inner_map.iter() {
        let analyzer_map = analyzer_setting.as_object().ok_or_else(|| {
            AnalyzerErrorKind::ParseFailure
                .with_error(anyhow::anyhow!("analyzer setting must be a map"))
        })?;

        // Initialize tokenizer.
        let tokenizer_map = analyzer_map["tokenizer"].as_object().ok_or_else(|| {
            AnalyzerErrorKind::ParseFailure
                .with_error(anyhow::anyhow!("tokenizer setting must be a map"))
        })?;

        let tokenizer_name = tokenizer_map["name"].as_str().ok_or_else(|| {
            AnalyzerErrorKind::ParseFailure
                .with_error(anyhow::anyhow!("tokenizer name must be a string"))
        })?;

        let tokenizer_args_bytes = if tokenizer_map.contains_key("args") {
            serde_json::to_vec(&tokenizer_map["args"])
                .map_err(|error| AnalyzerErrorKind::ParseFailure.with_error(error))?
        } else {
            Vec::new()
        };

        // Make text analyzer based on tokenizer.
        let mut analyzer = match TokenizerKind::from_str(tokenizer_name)
            .map_err(|error| AnalyzerErrorKind::ParseFailure.with_error(error))?
        {
            TokenizerKind::Lindera => create_lindera_analyzer(&tokenizer_args_bytes)
                .map_err(|error| AnalyzerErrorKind::ParseFailure.with_error(error))?,
            TokenizerKind::Ngram => create_ngram_analyzer(&tokenizer_args_bytes)
                .map_err(|error| AnalyzerErrorKind::ParseFailure.with_error(error))?,
            TokenizerKind::Raw => create_raw_analyzer(),
            TokenizerKind::Simple => create_simple_analyzer(),
            TokenizerKind::Whitespace => create_whitespace_analyzer(),
        };

        // Initialize filters.
        if analyzer_map.contains_key("filters") {
            let filters = analyzer_map["filters"].as_array().ok_or_else(|| {
                AnalyzerErrorKind::ParseFailure
                    .with_error(anyhow::anyhow!("filters must be an array"))
            })?;

            for filter in filters {
                let filter_setting_map = filter.as_object().ok_or_else(|| {
                    AnalyzerErrorKind::ParseFailure
                        .with_error(anyhow::anyhow!("filter setting must be a map"))
                })?;

                let filter_name = filter_setting_map["name"].as_str().ok_or_else(|| {
                    AnalyzerErrorKind::ParseFailure
                        .with_error(anyhow::anyhow!("filter name must be a string"))
                })?;

                let filter_args_bytes = if filter_setting_map.contains_key("args") {
                    serde_json::to_vec(&filter_setting_map["args"])
                        .map_err(|error| AnalyzerErrorKind::ParseFailure.with_error(error))?
                } else {
                    Vec::new()
                };

                match FilterKind::from_str(filter_name)
                    .map_err(|error| AnalyzerErrorKind::ParseFailure.with_error(error))?
                {
                    FilterKind::AlphaNumOnly => {
                        analyzer = analyzer.filter(create_alpha_num_only_filter());
                    }
                    FilterKind::AsciiFolding => {
                        analyzer = analyzer.filter(create_ascii_folding_filter());
                    }
                    FilterKind::LowerCase => {
                        analyzer = analyzer.filter(create_lower_case_filter());
                    }
                    FilterKind::RemoveLong => {
                        analyzer = analyzer.filter(
                            create_remove_long_filter(&filter_args_bytes).map_err(|error| {
                                AnalyzerErrorKind::ParseFailure.with_error(error)
                            })?,
                        );
                    }
                    FilterKind::Stemming => {
                        analyzer =
                            analyzer.filter(create_stemming_filter(&filter_args_bytes).map_err(
                                |error| AnalyzerErrorKind::ParseFailure.with_error(error),
                            )?);
                    }
                    FilterKind::StopWord => {
                        analyzer =
                            analyzer.filter(create_stop_word_filter(&filter_args_bytes).map_err(
                                |error| AnalyzerErrorKind::ParseFailure.with_error(error),
                            )?);
                    }
                }
            }
        }

        analyzers.insert(analyzer_name.to_string(), analyzer);
    }

    Ok(Analyzers {
        source: source.clone(),
        analyzers,
    })
}

pub fn register(manager: &TokenizerManager, analyzers: &Analyzers) -> Result<(), AnalyzerError> {
    for (name, analyzer) in analyzers.iter() {
        manager.register(name, analyzer.clone());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use tantivy::tokenizer::TokenizerManager;

    use crate::index::analyzer::{register, Analyzers};

    #[test]
    fn test_register() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("analyzers.json");

        let analyzers_bytes = fs::read(&path).unwrap();
        let analyzers = serde_json::from_slice::<Analyzers>(&analyzers_bytes).unwrap();

        let manager = TokenizerManager::default();

        let actual = register(&manager, &analyzers).unwrap();
        let expected = ();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_analyzers_deserialize() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("analyzers.json");

        let analyzers_bytes = fs::read(&path).unwrap();
        let analyzers = serde_json::from_slice::<Analyzers>(&analyzers_bytes).unwrap();

        let actual = analyzers.contains_key("default");
        let expected = true;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_analyzers_serialize() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("analyzers.json");

        let analyzers_bytes = fs::read(&path).unwrap();
        let analyzers = serde_json::from_slice::<Analyzers>(&analyzers_bytes).unwrap();

        let analyzers_bytes = serde_json::to_vec(&analyzers).unwrap();
        let actual = analyzers_bytes.len();
        let expected = 5018;
        assert_eq!(actual, expected);
    }
}
