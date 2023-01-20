use std::cell::Cell;
use std::fmt::Display;

use bytes::Bytes;
use tokio::net::TcpStream;

#[macro_use]
mod macros;
mod message_type;
use message_type::MessageType;

const MESSAGE_TYPE_BYTE_LENGTH: usize = 1;
const MESSAGE_USERNAME_LENGTH_BYTE_LENGTH: usize = 1;
const MESSAGE_BODY_LENGTH_BYTE_LENGTH: usize = 4;

#[derive(Debug)]
pub struct Error {
    pub description: String,
}

impl Error {
    pub fn new(description: &str) -> Self {
        Error {
            description: String::from(description),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}

pub type Result<T> = core::result::Result<T, Error>;

/// representing single message sent from client
///
/// constructed from TcpStream use [`Message::from_tcp_stream`]
pub struct Message {
    message_type: Bytes,
    _message_type: Cell<Option<MessageType>>,
    username_length: Bytes,
    username: Bytes,
    body_length: Bytes,
    body: Bytes,
}

impl Message {
    pub async fn from_tcp_stream(tcp_stream: &mut TcpStream) -> Result<Self> {
        let message_type = read_from_reader!(MESSAGE_TYPE_BYTE_LENGTH, tcp_stream, "type").await?;

        let username_length =
            read_from_reader!(MESSAGE_USERNAME_LENGTH_BYTE_LENGTH, tcp_stream, "length").await?;

        let username = read_from_reader!(username_length.len(), tcp_stream, "username").await?;

        let body_length =
            read_from_reader!(MESSAGE_BODY_LENGTH_BYTE_LENGTH, tcp_stream, "body length").await?;

        let body = read_from_reader!(body_length.len(), tcp_stream, "body").await?;

        Ok(Message {
            message_type,
            _message_type: Cell::new(None),
            username_length,
            username,
            body_length,
            body,
        })
    }

    pub fn get_message_type(&self) -> MessageType {
        let cache_message_type = self._message_type.get();
        if let Some(message_type) = cache_message_type {
            return message_type;
        }

        let message_type = MessageType::parse(&self.message_type).unwrap();
        self._message_type.set(Some(message_type));

        message_type
    }
}
