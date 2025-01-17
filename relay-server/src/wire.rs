#![allow(unused)]

use std::sync::LazyLock;

use bincode::{
    config::{Bounded, WithOtherLimit},
    Options,
};
use serde::{Deserialize, Serialize};

use crate::error::*;

const BINCODE_BYTE_LIMIT: u64 = 16 * 1024;

pub static BINCODE: LazyLock<WithOtherLimit<bincode::DefaultOptions, Bounded>> =
    LazyLock::new(|| bincode::DefaultOptions::new().with_limit(BINCODE_BYTE_LIMIT));

pub fn serialise<T: Serialize>(data: &T) -> Result<Vec<u8>> {
    let mut data = BINCODE.serialize(data)?;

    let mut final_data = (data.len() as u32).to_be_bytes().to_vec();
    final_data.append(&mut data);

    Ok(final_data)
}

pub fn deserialise<'a, T: Deserialize<'a>>(data: &'a [u8]) -> Result<T> {
    if data.len() < 4 {
        return Err(Error::InsufficientLenBytes);
    }

    let mut len_bytes = [0; 4];
    len_bytes.copy_from_slice(&data[..4]);

    let len = u32::from_be_bytes(len_bytes) as usize;

    let data: T = BINCODE.deserialize(&data[4..len])?;

    Ok(data)
}
