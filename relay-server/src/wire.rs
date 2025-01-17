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
