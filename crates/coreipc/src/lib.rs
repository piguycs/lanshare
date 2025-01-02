//! CoreIPC - Event driven IPC library for LANShare
//!
//! # TODO LIST
//! Make a REPL where I can send events to the client

use tokio::io::AsyncWriteExt;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;

use std::format as f;
use std::sync::Arc;
use std::{fs, io};

pub struct IpcServer {
    name: &'static str,
    socket: UnixListener,
    clients: Arc<Mutex<Vec<UnixStream>>>,
}

impl IpcServer {
    /// Creates a new server that initiates and manages a bi-directional data stream
    ///
    /// # Description
    /// The server acts as the initiator of the connection and maintains ownership of the socket.
    /// Since the data stream is bi-directional, both the server and client can send and receive data.
    pub fn create_server(name: &'static str) -> io::Result<Self> {
        Self::create_runtime_dir()?;

        let socket = UnixListener::bind(f!("/run/coreipc/{name}"))?;

        Ok(Self {
            socket,
            name,
            clients: Arc::default(),
        })
    }

    // send to all clients
    pub async fn broadcast(&self, pkt: &[u8]) {
        let mut clients = self.clients.lock().await;

        for stream in &mut *clients {
            if let Err(error) = stream.write_all(pkt).await {
                eprintln!("could not write to stream: {error}");
            }
        }
    }

    pub async fn run(&self) {
        loop {
            if let Ok((stream, _addr)) = self.socket.accept().await {
                let mut clients = self.clients.lock().await;
                clients.push(stream);
                drop(clients); // this is implied, but we do it anyways
            }
        }
    }

    fn create_runtime_dir() -> io::Result<()> {
        fs::create_dir_all("/run/coreipc")?;
        Ok(())
    }

    /// deleting the socket once the struct is dropped
    fn destroy_server(&self) -> io::Result<()> {
        let name = self.name;

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
