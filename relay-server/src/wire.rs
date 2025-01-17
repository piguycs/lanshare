use std::sync::LazyLock;

use bincode::{
    config::{Bounded, WithOtherLimit},
    Options,
};
use serde::{de::DeserializeOwned, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::error::*;

const BINCODE_BYTE_LIMIT: u64 = 16 * 1024;

pub static BINCODE: LazyLock<WithOtherLimit<bincode::DefaultOptions, Bounded>> =
    LazyLock::new(|| bincode::DefaultOptions::new().with_limit(BINCODE_BYTE_LIMIT));

#[instrument(skip(stream))]
pub async fn deserialise_stream<S, T>(stream: &mut S) -> Result<T>
where
    S: AsyncRead + Unpin,
    T: DeserializeOwned,
{
    let len = read_stream_len(stream).await?;

    let mut buf = vec![0; len];
    let data_len = stream
        .read_exact(&mut buf)
        .await
        .map_err(Error::WireError)?;

    if data_len != len {
        warn!(?data_len, ?len, "data len is not the same as expected len");
    }

    let data = BINCODE.deserialize(&buf)?;

    Ok(data)
}

pub async fn serialise_stream<S, T>(stream: &mut S, data: &T) -> Result
where
    S: AsyncWrite + Unpin,
    T: Serialize,
{
    let mut data = BINCODE.serialize(data)?;

    let len = data.len();

    let mut final_data = (len as u32).to_be_bytes().to_vec();
    final_data.append(&mut data);

    stream
        .write_all(&final_data)
        .await
        .map_err(Error::WireError)?;

    Ok(())
}

async fn read_stream_len<S>(stream: &mut S) -> Result<usize>
where
    S: AsyncRead + Unpin,
{
    let len = stream.read_u32().await.map_err(|error| {
        error!("error when getting len bytes: {error}");
        Error::InsufficientLenBytes
    })? as usize;

    Ok(len)
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use rstest::*;
    use serde::{Deserialize, Serialize};
    use std::io::Cursor;
    use std::pin::Pin;
    use tokio::io::{AsyncRead, AsyncWrite};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        field1: String,
        field2: i32,
    }

    // Mock async stream for testing
    struct MockStream {
        cursor: Cursor<Vec<u8>>,
    }

    impl MockStream {
        fn new(data: Vec<u8>) -> Self {
            Self {
                cursor: Cursor::new(data),
            }
        }
    }

    impl AsyncRead for MockStream {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            let n = std::io::Read::read(&mut self.cursor, buf.initialize_unfilled())?;
            buf.advance(n);
            std::task::Poll::Ready(Ok(()))
        }
    }

    impl AsyncWrite for MockStream {
        fn poll_write(
            mut self: Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<Result<usize, std::io::Error>> {
            let n = std::io::Write::write(&mut self.cursor, buf)?;
            std::task::Poll::Ready(Ok(n))
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), std::io::Error>> {
            std::task::Poll::Ready(Ok(()))
        }

        fn poll_shutdown(
            self: Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), std::io::Error>> {
            std::task::Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn test_read_stream_len() {
        let data = 42u32.to_be_bytes().to_vec();
        let mut stream = MockStream::new(data);
        let len = read_stream_len(&mut stream).await.unwrap();
        assert_eq!(len, 42);
    }

    #[rstest]
    #[tokio::test]
    async fn test_serialise_deserialise_round_trip() {
        let test_data = TestStruct {
            field1: "test".to_string(),
            field2: 123,
        };

        // Serialize
        let mut write_stream = MockStream::new(Vec::new());
        serialise_stream(&mut write_stream, &test_data)
            .await
            .unwrap();

        // Get the serialized data
        let serialized_data = write_stream.cursor.into_inner();

        // Deserialize
        let mut read_stream = MockStream::new(serialized_data);
        let deserialized: TestStruct = deserialise_stream(&mut read_stream).await.unwrap();

        assert_eq!(test_data, deserialized);
    }

    #[tokio::test]
    async fn test_deserialise_insufficient_len_bytes() {
        let mut stream = MockStream::new(vec![1, 2]); // Insufficient bytes for length
        let result: Result<TestStruct> = deserialise_stream(&mut stream).await;
        assert!(matches!(result, Err(Error::InsufficientLenBytes)));
    }

    #[tokio::test]
    async fn test_deserialise_invalid_data() {
        // Create invalid data with correct length prefix but invalid content
        let mut data = 4u32.to_be_bytes().to_vec();
        data.extend_from_slice(&[1, 2, 3, 4]); // Invalid bincode data
        let mut stream = MockStream::new(data);

        let result: Result<TestStruct> = deserialise_stream(&mut stream).await;
        assert!(matches!(result, Err(Error::BincodeError(_))));
    }

    #[tokio::test]
    async fn test_serialise_large_data() {
        // Create data that exceeds BINCODE_BYTE_LIMIT
        let large_string = "x".repeat(BINCODE_BYTE_LIMIT as usize + 1);
        let test_data = TestStruct {
            field1: large_string,
            field2: 123,
        };

        let mut stream = MockStream::new(Vec::new());
        let result = serialise_stream(&mut stream, &test_data).await;
        assert!(matches!(result, Err(Error::BincodeError(_))));
    }
}
