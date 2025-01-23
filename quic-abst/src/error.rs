use quinn::rustls;

pub type Result<T = (), E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    RustlsError(#[from] rustls::Error),
    #[error(transparent)]
    EncodeError(#[from] bincode::error::EncodeError),
    #[error(transparent)]
    DecodeError(#[from] bincode::error::DecodeError),
    #[error("stream ended")]
    StreamEnd,

    #[cfg(feature = "rcgen")]
    #[error(transparent)]
    CertGenError(#[from] rcgen::Error),
}
