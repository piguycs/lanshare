use std::io;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("quic error: {}", 0)]
    QuicError(#[from] QuicError),
}

#[derive(Debug, thiserror::Error)]
pub enum QuicError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    StartError(#[from] s2n_quic::provider::StartError),
}
