use super::{Error, Result};
use bytes::Bytes;

/// representing the MessageType in `Message` protocol first byte
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MessageType {
    /// indicated the message body as Text
    Text(String),
}

impl MessageType {
    /// parse u8 to [`MessageType`]
    ///
    /// return ['Ok(MessageType)'] if successfuls
    pub fn parse(bytes: u8, body: Bytes) -> Result<Self> {
        match bytes {
            1 => Ok(MessageType::Text(String::from_utf8(body.to_vec()).unwrap())),
            v => Err(Error::new(&format!("unsupported value: {}", v))),
        }
    }

    pub fn body_length(&self) -> u32 {
        match self {
            MessageType::Text(body) => body.len() as u32,
        }
    }

    pub fn as_bytes(&self) -> ([u8; 1], Bytes) {
        match self {
            MessageType::Text(body) => ([1], Bytes::from(body.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::MessageType;

    #[test]
    fn parse_succuss() {
        let body = String::from("test TEST");
        let body = Bytes::from(body);
        let res = MessageType::parse(1, body);

        assert_eq!(Ok(MessageType::Text(String::from("test TEST"))), res);
    }

    #[test]
    fn body_length_success() {
        let body = String::from("我I哒哒哒");
        let len = body.as_bytes().len() as u32;
        let body = Bytes::from(body);
        let res = MessageType::parse(1, body).unwrap();

        let body_len = res.body_length();

        assert_eq!(len, body_len);
    }
}
