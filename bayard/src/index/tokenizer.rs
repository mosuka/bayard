pub mod lindera;
pub mod ngram;
pub mod raw;
pub mod simple;
pub mod whitespace;

use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy)]
pub enum TokenizerFactoryErrorKind {
    CreateError,
    InvalidArgument,
    InvalidTokenizerKind,
}

impl TokenizerFactoryErrorKind {
    pub fn with_error<E>(self, source: E) -> TokenizerFactoryError
    where
        anyhow::Error: From<E>,
    {
        TokenizerFactoryError {
            kind: self,
            source: From::from(source),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("TokenizerFactoryError(kind={kind:?}, source={source})")]
pub struct TokenizerFactoryError {
    pub kind: TokenizerFactoryErrorKind,
    #[source]
    source: anyhow::Error,
}

impl TokenizerFactoryError {
    pub fn add_context<C>(self, ctx: C) -> Self
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        TokenizerFactoryError {
            kind: self.kind,
            source: self.source.context(ctx),
        }
    }

    pub fn kind(&self) -> TokenizerFactoryErrorKind {
        self.kind
    }
}

pub enum TokenizerKind {
    Lindera,
    Ngram,
    Raw,
    Simple,
    Whitespace,
}

impl FromStr for TokenizerKind {
    type Err = TokenizerFactoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lindera" => Ok(TokenizerKind::Lindera),
            "ngram" => Ok(TokenizerKind::Ngram),
            "raw" => Ok(TokenizerKind::Raw),
            "simple" => Ok(TokenizerKind::Simple),
            "whitespace" => Ok(TokenizerKind::Whitespace),
            _ => Err(TokenizerFactoryErrorKind::InvalidTokenizerKind
                .with_error(anyhow::format_err!("Unknown tokenizer kind: {}", s))),
        }
    }
}
