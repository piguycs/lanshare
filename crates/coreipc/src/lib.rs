//! CoreIPC - Event driven IPC library for LANShare
//!
//! # TODO LIST
//! - Make more maintainable and robust backend. I need to improve upon these points:
//!   - Right now, only one client is supported
//!   - The client can only connect once and cannot reconnect
//!   - The dev device is not allowed to capture packets without a client

mod server;

#[macro_use]
extern crate tracing;

use tokio::io::AsyncWriteExt;
use tokio::net::{UnixListener, UnixStream};

use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener as StdListener;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

pub struct IpcServer {
    socket_path: PathBuf,
    socket: UnixListener,
    client: Option<UnixStream>,
}

impl IpcServer {
    /// Creates a new server that initiates and manages a bi-directional data stream
    ///
    /// # Description
    /// The server acts as the initiator of the connection and maintains ownership of the socket.
    /// Since the data stream is bi-directional, both the server and client can send and receive data.
    pub fn create_server(name: &str) -> io::Result<Self> {
        let base_path = env::var("COREIPC_BASE_PATH").unwrap_or("/run/coreipc".to_string());
        let base_path = PathBuf::from(base_path);

        Self::create_runtime_dir(&base_path)?;

        let socket_path = base_path.join(name);

        dbg!("socket");
        dbg!(&socket_path);

        let listener = StdListener::bind(&socket_path)?;
        listener.set_nonblocking(true)?;

        let socket = UnixListener::from_std(listener)?;

        // Change the permissions to make it accessible by all users
        let permissions = fs::Permissions::from_mode(0o777);
        fs::set_permissions(&socket_path, permissions)?;

        Ok(Self {
            socket,
            socket_path,
            client: None,
        })
    }

    pub async fn broadcast(&mut self, pkt: &[u8]) {
        let len = pkt.len() as u16;
        info!("broadcast received with {} bytes", len);

        let res = match &mut self.client {
            Some(stream) => {
                if let Err(error) = stream.write_u16(len).await {
                    Err(error)
                } else {
                    stream.write_all(pkt).await
                }
            }
            None => todo!("encode state in the type"),
        };

        if let Err(error) = res {
            error!("could not write to stream {error}");
        }
    }

    pub async fn wait_for_client(&mut self) {
        if let Ok((stream, _addr)) = self.socket.accept().await {
            info!("added client");
            self.client = Some(stream);
        }
    }

    fn create_runtime_dir<P: AsRef<Path>>(base_path: P) -> io::Result<()> {
        if !base_path.as_ref().exists() {
            fs::create_dir_all(base_path)?;
        }

        Ok(())
    }

    /// deleting the socket once the struct is dropped
    fn destroy_server(&self) -> io::Result<()> {
        fs::remove_file(&self.socket_path)?;
        Ok(())
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        if let Err(error) = self.destroy_server() {
            eprintln!("error when destroying server: {error}");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn create_new_server() {
        let base_dir = format!("{}/target/test", env!("CARGO_MANIFEST_DIR"));
        unsafe {
            env::set_var("COREIPC_BASE_PATH", &base_dir);
        }

        let server_name = "hello_world";
        let path = format!("{base_dir}/{server_name}");

        {
            let _server = IpcServer::create_server(server_name).unwrap();
            assert!(std::path::Path::new(&path).exists());
        }

        // server is now dropped
        assert!(!std::path::Path::new(&path).exists());
    }
}
