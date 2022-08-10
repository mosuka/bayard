use std::{fmt, io::Read, str::FromStr};

use bytes::{Buf, BufMut, Bytes};
use foca::{Invalidates, Timer};
use time::OffsetDateTime;
use tracing::{debug, error, info};

#[derive(Debug, Clone, Copy)]
pub enum MessageErrorKind {
    ReadError,
    InvalidMessageKind,
    Invalidated,
}

impl MessageErrorKind {
    pub fn with_error<E>(self, source: E) -> MessageError
    where
        anyhow::Error: From<E>,
    {
        MessageError {
            kind: self,
            source: From::from(source),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("MessageError(kind={kind:?}, source={source})")]
pub struct MessageError {
    pub kind: MessageErrorKind,
    #[source]
    source: anyhow::Error,
}

impl MessageError {
    pub fn add_context<C>(self, ctx: C) -> Self
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        MessageError {
            kind: self.kind,
            source: self.source.context(ctx),
        }
    }

    pub fn kind(&self) -> MessageErrorKind {
        self.kind
    }
}

#[derive(Clone)]
pub enum Input<T> {
    Event(Timer<T>),
    Data(Bytes),
    Announce(T),
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum MessageKind {
    Unknown,
    CreateIndex,
    DeleteIndex,
    ModifyIndex,
}

impl FromStr for MessageKind {
    type Err = MessageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "unknown" => Ok(MessageKind::Unknown),
            "create_index" => Ok(MessageKind::CreateIndex),
            "delete_index" => Ok(MessageKind::DeleteIndex),
            "modify_index" => Ok(MessageKind::ModifyIndex),
            _ => Err(MessageErrorKind::InvalidMessageKind
                .with_error(anyhow::format_err!("Unknown message kind: {}", s))),
        }
    }
}

impl fmt::Debug for MessageKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MessageKind::Unknown => write!(f, "unknown"),
            MessageKind::CreateIndex => write!(f, "create_index"),
            MessageKind::DeleteIndex => write!(f, "delete_index"),
            MessageKind::ModifyIndex => write!(f, "modify_index"),
        }
    }
}

impl MessageKind {
    pub fn from_u8(n: u8) -> Result<Self, MessageError> {
        match n {
            0 => Ok(MessageKind::Unknown),
            3 => Ok(MessageKind::CreateIndex),
            4 => Ok(MessageKind::DeleteIndex),
            5 => Ok(MessageKind::ModifyIndex),
            _ => Err(MessageErrorKind::InvalidMessageKind
                .with_error(anyhow::format_err!("Unknown message kind: {:?}", n))),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            MessageKind::Unknown => 0,
            MessageKind::CreateIndex => 3,
            MessageKind::DeleteIndex => 4,
            MessageKind::ModifyIndex => 5,
        }
    }
}

pub const MESSAGE_NAME_FIELD: &str = "name";
pub const MESSAGE_METADATA_FIELD: &str = "metadata";

#[derive(Clone, PartialEq)]
pub struct Message {
    data: Vec<u8>,
}

impl Message {
    pub fn new(kind: MessageKind) -> Self {
        let data = &[];

        Message::with_body(kind, data)
    }

    pub fn with_body(kind: MessageKind, body: &[u8]) -> Self {
        let version = OffsetDateTime::now_utc().unix_timestamp();

        Message::with_body_version(kind, body, version)
    }

    pub fn with_body_version(kind: MessageKind, body: &[u8], version: i64) -> Self {
        let mut data: Vec<u8> = Vec::new();

        // Put version.
        data.put_i64(version);
        // Put kind.
        data.put_u8(kind.to_u8());
        // Put body length.
        data.put_u64(body.len() as u64);
        // Put body.
        data.append(&mut body.to_vec());

        Self { data }
    }

    pub fn from_bytes(src: impl Buf) -> Result<Self, MessageError> {
        let mut data: Vec<u8> = Vec::new();
        let mut reader = src.reader();
        let num = reader
            .read_to_end(&mut data)
            .map_err(|error| MessageErrorKind::ReadError.with_error(error))?;
        debug!(?num, "Read bytes.");

        Ok(Self { data })
    }

    pub fn version(&self) -> i64 {
        let mut buf = &self.data[0..8];
        buf.get_i64()
    }

    pub fn kind(&self) -> Result<MessageKind, MessageError> {
        let mut buf = &self.data[8..9];
        let kind = buf.get_u8();

        MessageKind::from_u8(kind)
    }

    pub fn body(&self) -> &[u8] {
        let mut buf = &self.data[9..17];
        let body_len = buf.get_u64();

        &self.data[17..17 + body_len as usize]
    }
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Message {{ kind: {:?}, body: {:?}, version: {} }}",
            self.kind(),
            self.body(),
            self.version(),
        )
    }
}

// Invalidation based on same key => higher version
impl Invalidates for Message {
    fn invalidates(&self, other: &Self) -> bool {
        let self_kind = match self.kind() {
            Ok(kind) => kind,
            Err(error) => {
                error!(?error, "Failed to get self message kind.");
                return false;
            }
        };
        let other_kind = match other.kind() {
            Ok(kind) => kind,
            Err(error) => {
                error!(?error, "Failed to get other message kind.");
                return false;
            }
        };

        let invalidated = self_kind == other_kind && self.version() > other.version();

        info!(
            ?invalidated,
            "Checking incoming data is invalidated or not."
        );

        invalidated
    }
}

impl AsRef<[u8]> for Message {
    fn as_ref(&self) -> &[u8] {
        self.data.as_slice()
    }
}

impl Default for Message {
    fn default() -> Self {
        Self::new(MessageKind::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use crate::cluster::message::{Message, MessageKind};

    #[test]
    fn new() {
        let kind = MessageKind::Unknown;
        let message = Message::new(kind);

        assert_eq!(message.kind().unwrap(), MessageKind::Unknown);
    }

    #[test]
    fn with_body() {
        let kind = MessageKind::Unknown;
        let body = b"test";
        let message = Message::with_body(kind, body);

        assert_eq!(message.kind().unwrap(), MessageKind::Unknown);
        assert_eq!(message.body(), body);
    }

    #[test]
    fn with_body_version() {
        let kind = MessageKind::Unknown;
        let body = b"test";
        let version = 1;
        let message = Message::with_body_version(kind, body, version);

        assert_eq!(message.kind().unwrap(), MessageKind::Unknown);
        assert_eq!(message.body(), body);
        assert_eq!(message.version(), version);
    }

    #[test]
    fn test_modify_index_message() {
        let modify_index_message_json_str = r#"
        {
            "name": "example",
            "index_metadata": {
                "schema": [
                    {
                        "name": "_id",
                        "type": "text",
                        "options": {
                            "indexing": {
                                "record": "basic",
                                "fieldnorms": true,
                                "tokenizer": "raw"
                            },
                            "stored": true,
                            "fast": false
                        }
                    },
                    {
                        "name": "_timestamp",
                        "type": "date",
                        "options": {
                            "indexed": true,
                            "fieldnorms": true,
                            "fast": "single",
                            "stored": true
                        }
                    },
                    {
                        "name": "url",
                        "type": "text",
                        "options": {
                            "indexing": {
                                "record": "freq",
                                "fieldnorms": false,
                                "tokenizer": "raw"
                            },
                            "stored": true,
                            "fast": false
                        }
                    },
                    {
                        "name": "name",
                        "type": "text",
                        "options": {
                            "indexing": {
                                "record": "position",
                                "fieldnorms": false,
                                "tokenizer": "default"
                            },
                            "stored": true,
                            "fast": false
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
                            "stored": true,
                            "fast": false
                        }
                    },
                    {
                        "name": "popularity",
                        "type": "u64",
                        "options": {
                            "indexed": true,
                            "fieldnorms": true,
                            "fast": "single",
                            "stored": true
                        }
                    },
                    {
                        "name": "category",
                        "type": "facet",
                        "options": {
                            "stored": true
                        }
                    },
                    {
                        "name": "publish_date",
                        "type": "date",
                        "options": {
                            "indexed": true,
                            "fieldnorms": true,
                            "fast": "single",
                            "stored": true
                        }
                    }
                ],
                "index_settings": {
                    "docstore_compression": "none",
                    "docstore_blocksize": 16384
                },
                "analyzers": {
                    "default": {
                        "filters":[
                            {
                                "args":{
                                    "length_limit": 40
                                },
                                "name": "remove_long"
                            },
                            {
                                "name": "ascii_folding"
                            },
                            {
                                "name": "lower_case"
                            }
                        ],
                        "tokenizer": {
                            "name": "simple"
                        }
                    },
                    "raw": {
                        "tokenizer": {
                            "name": "raw"
                        }
                    },
                    "whitespace": {
                        "tokenizer": {
                            "name": "whitespace"
                        }
                    }
                },
                "writer_threads": 1,
                "writer_mem_size": 500000000,
                "num_replicas": 2,
                "num_shards": 1,
                "shards": {
                    "shard_list": [
                        {
                            "id": "4UdJIxlX",
                            "state": "serving",
                            "version": 1660049618
                        },
                        {
                            "id": "QMcs7uHm",
                            "state": "draining",
                            "version": 1660049618
                        }
                    ]
                }
            }
        }
        "#;
        let modify_index_message_bytes = modify_index_message_json_str.as_bytes();
        let modify_index_message =
            serde_json::from_slice::<serde_json::Value>(modify_index_message_bytes).unwrap();
        let _modify_index_message_vec = serde_json::to_vec(&modify_index_message).unwrap();
    }
}
