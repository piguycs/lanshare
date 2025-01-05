//! CoreIPC - Event driven IPC library for LANShare
//!
//! # TODO LIST
//! - Make more maintainable and robust backend. I need to improve upon these points:
//! - Handle restarts of the backend
//! - ALlow recreating socket in case of pre-existing one

#[macro_use]
extern crate tracing;

pub mod client;
pub mod server;
mod wire;

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
