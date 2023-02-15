use super::{Error, Result};
use bytes::Bytes;

/// representing the MessageType in `Message` protocol first byte
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MessageType {
    Heart,
    /// indicated the message body as Text
    Text(String),
}

impl MessageType {
    /// parse u8 to [`MessageType`]
    ///
    /// return ['Ok(MessageType)'] if successfuls
    pub fn parse(value: u8, body: Option<Bytes>) -> Result<Self> {
        match value {
            0 => Ok(MessageType::Heart),
            1 => Ok(MessageType::Text(
                String::from_utf8(body.unwrap().to_vec()).unwrap(),
            )),
            other => Err(Error::new(&format!("unsupported value: {}", other))),
        }
    }

    pub fn body_length(&self) -> u32 {
        match self {
            MessageType::Heart => 0,
            MessageType::Text(body) => body.len() as u32,
        }
    }

    pub fn as_bytes(&self) -> Bytes {
        match self {
            MessageType::Heart => Bytes::new(),
            MessageType::Text(body) => Bytes::from(body.clone()),
        }
    }

    pub fn value(&self) -> u8 {
        match self {
            MessageType::Heart => 0,
            MessageType::Text(_) => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::MessageType;

    #[test]
    fn parse_text_succuss() {
        let body = String::from("test TEST");
        let body = Bytes::from(body);
        let res = MessageType::parse(1, Some(body));

        assert_eq!(Ok(MessageType::Text(String::from("test TEST"))), res);
    }

    #[test]
    fn parse_heart_success() {
        let res = MessageType::parse(0, None);

        assert_eq!(Ok(MessageType::Heart), res);
    }

    #[test]
    fn text_body_length_success() {
        let body = String::from("我I哒哒哒");
        let len = body.as_bytes().len() as u32;
        let body = Bytes::from(body);
        let res = MessageType::parse(1, Some(body)).unwrap();

        let body_len = res.body_length();

        assert_eq!(len, body_len);
    }

    #[test]
    fn heart_body_length() {
        let res = MessageType::parse(0, None).unwrap();

        let body_len = res.body_length();

        assert_eq!(0, body_len);
    }
}
