pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    DBusError(#[from] zbus::Error),
    #[error("relay connector error: {}", 0)]
    RelayError(#[from] relay_server::error::Error),
}
