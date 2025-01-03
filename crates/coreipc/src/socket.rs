use tokio::net::UnixListener;

use std::io;
use std::os::unix::net::UnixListener as StdUnixListener;

pub trait IntoSocket {
    fn into_socket(self) -> io::Result<UnixListener>;
}

impl IntoSocket for UnixListener {
    fn into_socket(self) -> io::Result<UnixListener> {
        Ok(self)
    }
}

impl IntoSocket for StdUnixListener {
    fn into_socket(self) -> io::Result<UnixListener> {
        UnixListener::from_std(self)
    }
}
