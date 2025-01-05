use std::io;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unnamed socket")]
    UnnamedSocket,
    #[error("bincode error: {}", 0)]
    BincodeError(#[from] bincode::Error),
    #[error("io error: {}", 0)]
    IoError(#[from] io::Error),
}
