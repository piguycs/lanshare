use std::io;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("quic error: {}", 0)]
    QuicError(#[from] QuicError),
    #[error("sqlite error: {}", 0)]
    SqliteError(#[from] rusqlite::Error),
    #[error("schema error: {}", 0)]
    SchemaError(rusqlite::Error),
    #[error("bincode error: {}", 0)]
    BincodeError(#[from] bincode::Error),
    #[error("server closed the connection prematurely")]
    PrematureClosure,
}

#[derive(Debug, thiserror::Error)]
pub enum QuicError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    StartError(#[from] s2n_quic::provider::StartError),
    #[error(transparent)]
    ConnectionError(#[from] s2n_quic::connection::Error),
}
