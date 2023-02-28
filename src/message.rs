//! wrap the data into 'message'
//!
//! >> availabe on **crate feature message** only.
//!
//! # Wrap data
//! the `message` will send and receive data with the format belowing:
//! 1 byte as message type at first, and then 1 byte as username length,
//! and then bytes as length of username, and then 4 bytes as body content length,
//! and then bytes as body
//!
//! |1 byte(indicated message type)|1 byte(indicated username length)|bytes, length depended in username length(indicated username who sending)|
//! |4 bytes(indicated body length)|bytes, length depended in body content length(indicated body which communicating)|

use std::fmt::Display;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

mod message_type;
pub use message_type::MessageType;

const MESSAGE_TYPE_BYTE_LENGTH: usize = 1;
const MESSAGE_USERNAME_LENGTH_BYTE_LENGTH: usize = 1;
const MESSAGE_BODY_LENGTH_BYTE_LENGTH: usize = 4;
const DEFAULT_BUFFER_CAPACITY: usize = 512;

#[derive(Debug, PartialEq, Eq)]
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

/// representing single message
///
/// constructed from TcpStream use [`Message::from_tcp_stream`]
pub struct Message {
    pub message_type: MessageType,
    username_length: u8,
    pub username: String,
    _body_length: u32,
}

impl Message {
    pub fn new(message_type: MessageType, username: String) -> Self {
        let username_length = username.len() as u8;
        let _body_length = message_type.body_length();

        Message {
            message_type,
            username_length,
            username,
            _body_length,
        }
    }

    pub async fn send(
        tcp_stream: &mut (impl AsyncWriteExt + std::marker::Unpin),
        message: Self,
    ) -> Result<()> {
        let mut bytes = message.to_bytes();

        tcp_stream.write_buf(&mut bytes).await.unwrap();
        Ok(())
    }

    pub async fn read_from(stream: &mut (impl AsyncReadExt + Unpin)) -> Result<Option<Self>> {
        let mut bytes = BytesMut::with_capacity(DEFAULT_BUFFER_CAPACITY);

        let len = stream.read_buf(&mut bytes).await.map_err(|e| {
            let description = format!("read message failure: {}", e.kind());
            Error {
                description: String::from(description),
            }
        })?;

        if len == 0 {
            return Ok(None);
        }

        Message::varify_len(MESSAGE_TYPE_BYTE_LENGTH, bytes.len())?;
        let message_type = bytes.get_u8();
        Message::varify_len(MESSAGE_USERNAME_LENGTH_BYTE_LENGTH, bytes.len())?;
        let username_len = bytes.get_u8();
        Message::varify_len(username_len as usize, bytes.len())?;
        let username = bytes.split_to(username_len as usize);
        let username = String::from_utf8(username.to_vec()).unwrap();
        Message::varify_len(MESSAGE_BODY_LENGTH_BYTE_LENGTH, bytes.len())?;
        let body_len = bytes.get_u32();
        Message::varify_len(body_len as usize, bytes.len())?;
        let body = bytes.split_to(body_len as usize);

        Ok(Some(Message {
            message_type: MessageType::parse(message_type, Some(body.freeze())).unwrap(),
            username_length: username_len,
            username,
            _body_length: body_len,
        }))

        // MessageType::parse(message_type, body);
    }

    /// get the body of message
    /// return Some(body) if message_type is Text otherwise None
    pub fn get_body(&self) -> Option<&String> {
        match &self.message_type {
            MessageType::Text(body) => Some(body),
            _ => None
        }
    }

    fn varify_len(expect_len: usize, actual_len: usize) -> Result<()> {
        if actual_len < expect_len {
            Err(Error {
                description: String::from(format!(
                    "actual length {} lower than expect length {}",
                    actual_len, expect_len
                )),
            })
        } else {
            Ok(())
        }
    }

    fn to_bytes(self) -> Bytes {
        let body = self.message_type.as_bytes();
        let username = Bytes::from(self.username.clone());
        let username_length = self.username_length;
        let body_length = body.len() as u32;

        let capacity_length = MESSAGE_TYPE_BYTE_LENGTH + body.len() + username.len() + 2;
        let mut bytes = BytesMut::with_capacity(capacity_length);
        bytes.put_u8(self.message_type.value());
        bytes.put_u8(username_length);
        bytes.put(username);
        bytes.put_u32(body_length);
        bytes.put(body);

        bytes.freeze()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_bytes_success() {
        let username = String::from("dvorak");
        let body = String::from("test body");
        let message_type = MessageType::Text(body.clone());

        let message = Message::new(message_type, username.clone());
        let bytes = message.to_bytes();

        let expected_username_len = username.as_bytes().len() as u8;
        let expected_username = username.as_bytes();
        let mut expected_bytes = BytesMut::with_capacity(bytes.len());
        expected_bytes.put_u8(1u8);
        expected_bytes.put_u8(expected_username_len);
        expected_bytes.put(expected_username);
        expected_bytes.put_u32(body.as_bytes().len() as u32);
        expected_bytes.put(body.as_bytes());

        assert_eq!(expected_bytes, bytes);
    }

    #[tokio::test]
    async fn read_from_duplex_stream() {
        let (mut client, mut server) = tokio::io::duplex(64);

        let username = String::from("dvorak");
        let body = String::from("test body");
        let message_type = MessageType::Text(body.clone());

        let message = Message::new(message_type, username.clone());
        let mut message_bytes = message.to_bytes();
        let expected_len = message_bytes.len();

        let len = client.write_buf(&mut message_bytes).await.unwrap();
        assert_eq!(len, expected_len);

        let message = Message::read_from(&mut server).await.unwrap().unwrap();

        assert_eq!(message._body_length as usize, body.len());
        assert_eq!(message.message_type, MessageType::Text(body));
        assert_eq!(message.username_length as usize, username.len());
        assert_eq!(message.username, username);
    }

    #[test]
    fn get_body_has_body() {
        let body = String::from("message body");
        let username = String::from("uuuusername");
        let message = Message::new(MessageType::Text(body.clone()), username);

        let body_value = message.get_body();
        assert_eq!(Some(&body), body_value);
    }

    #[test]
    fn get_body_has_no_body() {
        let username = String::from("uuuusername");
        let message = Message::new(MessageType::Login, username);

        let body_value = message.get_body();
        assert_eq!(None, body_value);
    }
}
