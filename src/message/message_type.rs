use super::{Error, Result};
use bytes::Bytes;

#[derive(Clone, Copy)]
pub enum MessageType {
    Text,
}

impl MessageType {
    pub fn parse(bytes: &Bytes) -> Result<Self> {
        const BYTE_LENGTH: usize = 1;
        let len = bytes.len();
        if len != BYTE_LENGTH {
            return Err(Error::new(&format!("incorrect length: {}", len)));
        }

        match bytes.first().unwrap() {
            &1 => Ok(MessageType::Text),
            &v => Err(Error::new(&format!("unsupported value: {}", v))),
        }
    }
}
