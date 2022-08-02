pub mod all;
pub mod boolean;
pub mod boost;
pub mod fuzzy_term;
pub mod phrase;
pub mod query_string;
pub mod range;
pub mod regex;
pub mod term;

use std::fmt;

use tantivy::{
    query::{Query, QueryClone},
    Searcher,
};

use crate::proto::index::{query::Kind as ProtoQueryKind, Query as ProtoQuery};

use self::{
    all::create_all_query,
    boolean::{create_boolean_query, BooleanQueryArgs},
    boost::{create_boost_query, BoostQueryArgs},
    fuzzy_term::{create_fuzzy_term_query, FuzzyTermQueryArgs},
    phrase::{create_phrase_query, PhraseQueryArgs},
    query_string::{create_query_string_query, QueryStringQueryArgs},
    range::{create_range_query, RangeQueryArgs},
    regex::{create_regex_query, RegexQueryArgs},
    term::{create_term_query, TermQueryArgs},
};

#[derive(Debug, Clone, Copy)]
pub enum QueryFactoryErrorKind {
    CreateError,
    InvalidArgument,
}

impl QueryFactoryErrorKind {
    pub fn with_error<E>(self, source: E) -> QueryFactoryError
    where
        anyhow::Error: From<E>,
    {
        QueryFactoryError {
            kind: self,
            source: From::from(source),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("QueryFactoryError(kind={kind:?}, source={source})")]
pub struct QueryFactoryError {
    pub kind: QueryFactoryErrorKind,
    #[source]
    source: anyhow::Error,
}

impl QueryFactoryError {
    pub fn add_context<C>(self, ctx: C) -> Self
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        QueryFactoryError {
            kind: self.kind,
            source: self.source.context(ctx),
        }
    }

    pub fn kind(&self) -> QueryFactoryErrorKind {
        self.kind
    }
}

pub fn create_query(
    query: &ProtoQuery,
    searcher: &Searcher,
) -> Result<Box<dyn Query>, QueryFactoryError> {
    let kind = ProtoQueryKind::from_i32(query.kind).ok_or_else(|| {
        QueryFactoryErrorKind::InvalidArgument
            .with_error(anyhow::anyhow!("Undefined query kind {}", query.kind))
    })?;

    let query_obj = match kind {
        ProtoQueryKind::All => create_all_query().box_clone(),
        ProtoQueryKind::Boolean => {
            let args = serde_json::from_slice::<BooleanQueryArgs>(query.options.as_slice())
                .map_err(|e| QueryFactoryErrorKind::InvalidArgument.with_error(e))?;
            create_boolean_query(&args, searcher)?.box_clone()
        }
        ProtoQueryKind::Boost => {
            let args = serde_json::from_slice::<BoostQueryArgs>(query.options.as_slice())
                .map_err(|e| QueryFactoryErrorKind::InvalidArgument.with_error(e))?;
            create_boost_query(&args, searcher)?.box_clone()
        }
        ProtoQueryKind::FuzzyTerm => {
            let args = serde_json::from_slice::<FuzzyTermQueryArgs>(query.options.as_slice())
                .map_err(|e| QueryFactoryErrorKind::InvalidArgument.with_error(e))?;
            create_fuzzy_term_query(&args, searcher)?.box_clone()
        }
        ProtoQueryKind::Phrase => {
            let args = serde_json::from_slice::<PhraseQueryArgs>(query.options.as_slice())
                .map_err(|e| QueryFactoryErrorKind::InvalidArgument.with_error(e))?;
            create_phrase_query(&args, searcher)?.box_clone()
        }
        ProtoQueryKind::QueryString => {
            let args = serde_json::from_slice::<QueryStringQueryArgs>(query.options.as_slice())
                .map_err(|e| QueryFactoryErrorKind::InvalidArgument.with_error(e))?;
            create_query_string_query(&args, searcher)?
        }
        ProtoQueryKind::Range => {
            let args = serde_json::from_slice::<RangeQueryArgs>(query.options.as_slice())
                .map_err(|e| QueryFactoryErrorKind::InvalidArgument.with_error(e))?;
            create_range_query(&args, searcher)?.box_clone()
        }
        ProtoQueryKind::Regex => {
            let args = serde_json::from_slice::<RegexQueryArgs>(query.options.as_slice())
                .map_err(|e| QueryFactoryErrorKind::InvalidArgument.with_error(e))?;
            create_regex_query(&args, searcher)?.box_clone()
        }
        ProtoQueryKind::Term => {
            let args = serde_json::from_slice::<TermQueryArgs>(query.options.as_slice())
                .map_err(|e| QueryFactoryErrorKind::InvalidArgument.with_error(e))?;
            create_term_query(&args, searcher)?.box_clone()
        }
        _ => {
            return Err(QueryFactoryErrorKind::InvalidArgument
                .with_error(anyhow::anyhow!("Unsupported query kind {:?}", kind)))
        }
    };

    Ok(query_obj)
}
