//! CoreIPC - Event driven IPC library for LANShare
//!
//! # TODO LIST
//! Make a REPL where I can send events to the client

#[macro_use]
extern crate tracing;

use tokio::io::AsyncWriteExt;
use tokio::net::{UnixListener, UnixStream};

use std::format as f;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener as StdListener;
use std::{fs, io};

pub struct IpcServer {
    name: String,
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
        Self::create_runtime_dir()?;

        let socket_path = format!("/run/coreipc/{}", name);

        let listener = StdListener::bind(&socket_path)?;
        listener.set_nonblocking(true)?;

        let socket = UnixListener::from_std(listener)?;

        // Change the permissions to make it accessible by all users
        let permissions = fs::Permissions::from_mode(0o777);
        fs::set_permissions(&socket_path, permissions)?;

        Ok(Self {
            socket,
            name: name.to_string(),
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

    fn create_runtime_dir() -> io::Result<()> {
        fs::create_dir_all("/run/coreipc")?;
        Ok(())
    }

    /// deleting the socket once the struct is dropped
    fn destroy_server(&self) -> io::Result<()> {
        let name = &self.name;

        fs::remove_file(f!("/run/coreipc/{name}"))?;
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

    #[test]
    fn create_new_server() {
        let server_name = "hello_world";
        let path = format!("/run/coreipc/{}", server_name);

        {
            let _server = IpcServer::create_server(server_name).unwrap();
            assert!(std::path::Path::new(&path).exists());
        }

        // server is now dropped
        assert!(!std::path::Path::new(&path).exists());
    }
}
