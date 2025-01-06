//! CoreIPC Server - The owner of the data stream

use tokio::io::AsyncReadExt;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;

use std::collections::HashMap;
use std::io::{self};
use std::sync::Arc;
use std::{env, path::PathBuf};

use crate::IntoSocket;

pub const COREIPC_RUNTIME_DIR: &str = "COREIPC_RUNTIME_DIR";

#[derive(Debug)]
struct ClientStream {
    stream: UnixStream,
    socket: UnixListener,
}

impl ClientStream {
    async fn create(stream: &mut UnixStream) -> io::Result<()> {
        let len = stream.read_u16().await? as usize;

        let mut buf = vec![0; len];

        let data = stream.read_exact(&mut buf).await?;

        assert_eq!(data, len);

        Ok(())
    }
}

type ClientsArc = Arc<RwLock<HashMap<u16, UnixStream>>>;

#[derive(Debug)]
pub struct Ipc {
    socket: UnixListener,
    clients: ClientsArc,
}

impl Ipc {
    /// Creates a new server that initiates and manages a data stream
    ///
    /// # Description
    /// The server acts as the initiator of the connection and maintains ownership of the socket.
    /// Since the data stream is bi-directional, both the server and client can send and receive data.
    ///
    /// Creates a socket in `/run/coreipc/{name}` or in `$COREIPC_RUNTIME_DIR/{name}` if the
    /// enviorenment variable is set
    ///
    /// # Example
    /// ```rust
    /// let server = Ipc::create_server()?;
    /// let _ = server.run().await;
    /// ```
    #[instrument]
    pub fn create_server(name: &str) -> io::Result<Self> {
        info!("creating server using name");
        let socket_path = Self::get_socket_path(name);

        let socket = UnixListener::bind(&socket_path)?;
        debug!(?socket);

        Ok(Self {
            socket,
            clients: Arc::default(),
        })
    }

    /// Creates a new server that initiates and manages a data stream. Uses an existing
    /// UnixListener from either the standard library or tokio::net instead of creating a new one
    ///
    /// this is infailable when using [tokio::net::UnixListener] because no type conversion is
    /// necessary. It can fail when using a [std::os::unix::net::UnixListener].
    ///
    /// # Description
    /// The server acts as the initiator of the connection and maintains ownership of the socket.
    /// Since the data stream is bi-directional, both the server and client can send and receive data.
    ///
    /// # Example
    /// ```rust
    /// let socket = tokio::net::UnixListener::bind("ipc.sock");
    /// let server = Ipc::from_socketr().unwrap();
    /// let _ = server.run().await;
    /// ```
    #[instrument(skip(unix_socket))]
    pub fn from_socket<S: IntoSocket>(unix_socket: S) -> io::Result<Self> {
        info!("creating server using socket");
        let socket = unix_socket.into_socket()?;
        debug!(?socket);

        Ok(Self {
            socket,
            clients: Arc::default(),
        })
    }

    #[instrument(skip(self))]
    pub async fn run(&self) {
        let clients = Arc::clone(&self.clients);

        match self.socket.accept().await {
            Ok((mut stream, _addr)) => {
                let client_id = rand::random();
                let mut clients_w = clients.write().await;

                // TODO: remove unwrap
                ClientStream::create(&mut stream).await.unwrap();

                clients_w.insert(client_id, stream);

                let clients = Arc::clone(&clients);

                tokio::spawn(Self::handle_client(client_id, clients));
            }
            Err(error) => error!("could not accept connection: {error}"),
        }
    }

    #[instrument]
    async fn handle_client(client_id: u16, clients: ClientsArc) {
        debug!(?client_id, ?clients);
        todo!();
    }

    #[instrument]
    fn get_socket_path(name: &str) -> PathBuf {
        let default = "/run/coreipc".into();
        let path = env::var_os(COREIPC_RUNTIME_DIR).unwrap_or(default);

        let base_dir = PathBuf::from(path);
        debug!(?base_dir);

        base_dir.join(name)
    }
}

#[cfg(test)]
mod unit_test {
    use super::*;

    #[test]
    fn socket_path_custom() {
        let name = "test.sock";
        let sock_dir = "/tmp";

        temp_env::with_var(COREIPC_RUNTIME_DIR, Some(sock_dir), || {
            let socket_path = Ipc::get_socket_path(name);
            assert_eq!(socket_path.as_os_str(), "/tmp/test.sock");
        });
    }

    #[test]
    fn socket_path() {
        let socket_path = Ipc::get_socket_path("test.sock");
        assert_eq!(socket_path.as_os_str(), "/run/coreipc/test.sock");
    }

    #[tokio::test]
    async fn server_creation() -> std::io::Result<()> {
        // this ensures cleanup of the socket after test
        // you can verify this with `ls /tmp/.tmp*` before and after
        let socket = tempfile::Builder::new().make(|e| UnixListener::bind(e))?;
        Ipc::from_socket(socket.into_file())?;

        Ok(())
    }

    #[tokio::test]
    async fn server_creation_std() -> std::io::Result<()> {
        use std::os::unix::net::UnixListener;

        let socket = tempfile::Builder::new().make(|e| UnixListener::bind(e))?;
        Ipc::from_socket(socket.into_file())?;

        Ok(())
    }
}
