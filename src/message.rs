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
use tokio::{io::{AsyncWriteExt}, net::TcpStream};
use tokio::net::tcp::{ReadHalf, WriteHalf};

#[macro_use]
mod macros;
mod message_type;
pub use message_type::MessageType;

const MESSAGE_TYPE_BYTE_LENGTH: usize = 1;
const MESSAGE_USERNAME_LENGTH_BYTE_LENGTH: usize = 1;
const MESSAGE_BODY_LENGTH_BYTE_LENGTH: usize = 4;

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
    body_length: u32,
}

impl Message {
    pub fn new(message_type: MessageType, username: String) -> Self {
        let username_length = username.len() as u8;
        let body_length = message_type.body_length();

        Message {
            message_type,
            username,
            username_length,
            body_length,
        }
    }

    pub async fn send(tcp_stream: &mut WriteHalf<'_>, message: Message) -> Result<()> {
        let mut bytes = message.to_bytes();

        let res = tcp_stream.write_buf(&mut bytes).await;

        if let Ok(size) = res {
            if size == bytes.len() {
                return Ok(());
            } else {
                return Err(Error::new("send failure"));
            }
        } else {
            return Err(Error::new(res.unwrap_err().kind().to_string().as_str()));
        }
    }

    pub async fn from_tcp_stream(tcp_stream: &mut TcpStream) -> Result<Self> {

        let (mut read_half, _) = tcp_stream.split();

        return Self::from_read_half(&mut read_half).await;
    }

    pub async fn from_read_half(read_half: &mut ReadHalf<'_>) -> Result<Self> {
        let message_type = read_from_reader!(MESSAGE_TYPE_BYTE_LENGTH, read_half, "type").await?;

        let username_length =
            read_from_reader!(MESSAGE_USERNAME_LENGTH_BYTE_LENGTH, read_half, "length").await?;

        let username = read_from_reader!(username_length.len(), read_half, "username").await?;

        let body_length =
            read_from_reader!(MESSAGE_BODY_LENGTH_BYTE_LENGTH, read_half, "body length").await?;

        let body = read_from_reader!(body_length.len(), read_half, "body").await?;

        let message = Message {
            message_type: Message::get_message_type(message_type, body),
            username_length: Message::get_username_length(username_length),
            username: Message::get_username(username),
            body_length: Message::get_body_length(body_length),
        };

        if message.varify() {
            Ok(message)
        } else {
            Err(Error::new("cannot get the correct message"))
        }
    }

    pub fn varify(&self) -> bool {
        self.username_length as usize == self.username.len()
            && self.message_type.body_length() == self.body_length
    }

    fn get_message_type(byte: Bytes, body: Bytes) -> MessageType {
        let message_type = MessageType::parse(*byte.first().unwrap(), body).unwrap();

        message_type
    }

    fn get_username_length(bytes: Bytes) -> u8 {
        let mut b = bytes;
        b.get_u8()
    }

    fn get_username(bytes: Bytes) -> String {
        String::from_utf8(bytes.into()).unwrap()
    }

    fn get_body_length(bytes: Bytes) -> u32 {
        let mut b = bytes;
        b.get_u32()
    }

    fn to_bytes(self) -> Bytes {
        let (message_type, body) = self.message_type.as_bytes();
        let username = Bytes::from(self.username.clone());
        let username_length = self.username_length;
        let body_length = body.len() as u32;

        let capacity_length = message_type.len() + body.len() + username.len() + 2;
        let mut bytes = BytesMut::with_capacity(capacity_length);
        bytes.put(&message_type[..]);
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
    fn get_message_type_success() {
        let test_body: &str = "test Test";
        let b = Bytes::from(&[1u8][..]);
        let body = Bytes::from(test_body);

        let res = Message::get_message_type(b, body);

        assert_eq!(res, MessageType::Text(String::from(test_body)));
    }

    #[test]
    fn get_unername_length_success() {
        const LEN: u8 = 123u8;
        let byte = Bytes::from(&[LEN][..]);
        let res = Message::get_username_length(byte);

        assert_eq!(LEN, res);
    }

    #[test]
    fn get_unername_success() {
        const USERNAME: &str = "DVORAK";
        let bytes = Bytes::from(USERNAME);

        let res = Message::get_username(bytes);

        assert_eq!(USERNAME, res);
    }

    #[test]
    fn get_body_length() {
        let bytes = Bytes::from(&[0u8, 0u8, 1u8, 1u8][..]);

        let res = Message::get_body_length(bytes);

        assert_eq!(257, res);
    }

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
}
