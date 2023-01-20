/// pull and return bytes with specified number
///
/// The `read_from_reader!` macro must be used inside of async function
///
/// # Example:
/// ```
/// use tokio::net::TcpListener;
///
/// let tcp = TcpListener::bind("127.0.0.1:9999").await.unwrap();
/// let mut tcp_stream = tcp.accept().await.unwrap();
/// const READ_LENGTH: usize = 1;
///
/// let data = read_from_reader!(READ_LENGTH, tcp_stream, "error message type");
///
/// assert_eq!(READ_LENGTH, data.len());
/// ```
///
/// # Parameters
///
/// 'len': how many bytes would read
///
/// 'reader': read source, must implement `tokio::io::AsyncReaderExt`
///
/// 'err_type': error description type of `Message`, example: `byte`
///
/// # Return
/// [`Ok(bytes::Bytes)`] or [`Err(Error)`], 
/// error ocurred either if $reader.read_buf returning Err or real length read from $reader is not equeal to $len
macro_rules! read_from_reader {
    ($len: expr, $reader: expr, $err_type: expr) => {
        async {
            use crate::message::Error;
            use bytes::BytesMut;
            use tokio::io::AsyncReadExt;

            let mut data = BytesMut::with_capacity($len);
            let len = $reader.read_buf(&mut data).await.map_err(|e| {
                let description = format!("read message {} failure: {}", $err_type, e.kind());
                Error {
                    description: String::from(description),
                }
            })?;

            if len != data.capacity() {
                return Err(Error {
                    description: format!(
                        "read message {} failure: incorrect length {}",
                        $err_type, len
                    ),
                });
            }

            Ok(data.freeze())
        }
    };
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    #[tokio::test]
    async fn read_from_reader_should_success() {
        const LEN: usize = 1;
        const ERROR_TYPE: &str = "TEST";
        let mut reader = Cursor::new(vec![1u8]);

        let data = read_from_reader!(LEN, reader, ERROR_TYPE).await.unwrap();

        assert_eq!(LEN, data.len());

        let first_item = data.first().unwrap();
        assert_eq!(1u8, *first_item);
    }

    #[tokio::test]
    async fn read_from_reader_should_failure() {
        const LEN: usize = 2;
        const ERROR_TYPE: &str = "TEST";
        let mut reader = Cursor::new(vec![1u8]);

        let data = read_from_reader!(LEN, reader, ERROR_TYPE)
            .await
            .unwrap_err();

        assert_eq!(
            data.description,
            format!(
                "read message {} failure: incorrect length {}",
                ERROR_TYPE, 1
            )
        );
    }
}
