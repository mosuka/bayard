pub mod alpha_num_only;
pub mod ascii_folding;
pub mod lower_case;
pub mod remove_long;
pub mod stemming;
pub mod stop_word;

use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy)]
pub enum FilterFactoryErrorKind {
    InvalidArgument,
    InvalidFilterKind,
}

impl FilterFactoryErrorKind {
    pub fn with_error<E>(self, source: E) -> FilterFactoryError
    where
        anyhow::Error: From<E>,
    {
        FilterFactoryError {
            kind: self,
            source: From::from(source),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("FilterFactoryError(kind={kind:?}, source={source})")]
pub struct FilterFactoryError {
    pub kind: FilterFactoryErrorKind,
    #[source]
    source: anyhow::Error,
}

impl FilterFactoryError {
    pub fn add_context<C>(self, ctx: C) -> Self
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        FilterFactoryError {
            kind: self.kind,
            source: self.source.context(ctx),
        }
    }

    pub fn kind(&self) -> FilterFactoryErrorKind {
        self.kind
    }
}

pub enum FilterKind {
    AlphaNumOnly,
    AsciiFolding,
    LowerCase,
    RemoveLong,
    Stemming,
    StopWord,
}

impl FromStr for FilterKind {
    type Err = FilterFactoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "alpha_num_only" => Ok(FilterKind::AlphaNumOnly),
            "ascii_folding" => Ok(FilterKind::AsciiFolding),
            "lower_case" => Ok(FilterKind::LowerCase),
            "remove_long" => Ok(FilterKind::RemoveLong),
            "stemming" => Ok(FilterKind::Stemming),
            "stop_word" => Ok(FilterKind::StopWord),
            _ => Err(FilterFactoryErrorKind::InvalidFilterKind
                .with_error(anyhow::format_err!("Unknown filter kind: {}", s))),
        }
    }
}
