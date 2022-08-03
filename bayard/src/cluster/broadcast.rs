use std::{collections::HashMap, fmt};

use bytes::Buf;
use foca::BroadcastHandler;
use tracing::{debug, error, info};

use super::message::{Message, MessageKind};

#[derive(Debug, Clone, Copy)]
pub enum MessageHandlerErrorKind {
    SendError,
    ReceiveError,
}

impl MessageHandlerErrorKind {
    pub fn with_error<E>(self, source: E) -> MessageHandlerError
    where
        anyhow::Error: From<E>,
    {
        MessageHandlerError {
            kind: self,
            source: From::from(source),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("MessageHandlerError(kind={kind:?}, source={source})")]
pub struct MessageHandlerError {
    pub kind: MessageHandlerErrorKind,
    #[source]
    source: anyhow::Error,
}

impl MessageHandlerError {
    pub fn add_context<C>(self, ctx: C) -> Self
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        MessageHandlerError {
            kind: self.kind,
            source: self.source.context(ctx),
        }
    }

    pub fn kind(&self) -> MessageHandlerErrorKind {
        self.kind
    }
}

pub struct MessageHandler {
    tx_sender_message: tokio::sync::watch::Sender<Message>,
    latest_message: HashMap<MessageKind, i64>,
}

impl MessageHandler {
    pub fn new(tx_sender_message: tokio::sync::watch::Sender<Message>) -> Self {
        Self {
            tx_sender_message,
            latest_message: HashMap::new(),
        }
    }
}

impl<T> BroadcastHandler<T> for MessageHandler {
    type Broadcast = Message;

    type Error = MessageHandlerError;

    fn receive_item(&mut self, data: impl Buf) -> Result<Option<Self::Broadcast>, Self::Error> {
        let message = match Message::from_bytes(data)
            .map_err(|e| MessageHandlerErrorKind::ReceiveError.with_error(e))
        {
            Ok(message) => message,
            Err(error) => {
                error!(?error, "Failed to read data.");
                return Ok(None);
            }
        };

        let kind = match message
            .kind()
            .map_err(|e| MessageHandlerErrorKind::ReceiveError.with_error(e))
        {
            Ok(kind) => kind,
            Err(error) => {
                error!(?error, "Failed to get message kind.");
                return Ok(None);
            }
        };

        let version = message.version();

        match self.latest_message.get(&kind) {
            Some(ver) => {
                if ver >= &version {
                    debug!(?kind, ?version, "Received message is invalidated.");
                    return Ok(None);
                } else {
                    info!(?kind, ?version, "Received higher version message.");
                }
            }
            None => {
                info!(?kind, ?version, "Received a new message.");
            }
        }

        self.latest_message.insert(kind, version);

        match self
            .tx_sender_message
            .send(message.clone())
            .map_err(|e| MessageHandlerErrorKind::SendError.with_error(e))
        {
            Ok(_) => {
                info!(?kind, ?version, "Sent message to message sender.");
            }
            Err(e) => {
                error!(?e, "Failed to send message to message sender.");
                return Ok(None);
            }
        }

        Ok(Some(message))
    }
}
