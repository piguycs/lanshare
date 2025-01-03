use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;

use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
use std::{env, io, path::PathBuf};

use crate::socket::IntoSocket;

pub const COREIPC_RUNTIME_DIR: &str = "COREIPC_RUNTIME_DIR";

type ClientsArc = Arc<RwLock<HashMap<u32, UnixStream>>>;

#[derive(Debug)]
pub struct Ipc<T> {
    socket: UnixListener,
    clients: ClientsArc,

    data_type: PhantomData<T>,
}

impl<T> Ipc<T> {
    #[instrument]
    pub fn create_server(name: &str) -> io::Result<Self> {
        let socket_path = Self::get_socket_path(name);

        let socket = UnixListener::bind(&socket_path)?;
        debug!(?socket);

        Ok(Self {
            socket,
            clients: Arc::default(),

            data_type: PhantomData,
        })
    }

    #[instrument(skip(socket))]
    pub fn from_socket<S: IntoSocket>(socket: S) -> io::Result<Self> {
        let socket = socket.into_socket()?;

        Ok(Self {
            socket,
            clients: Arc::default(),

            data_type: PhantomData,
        })
    }

    pub async fn run(&self) {
        let clients = Arc::clone(&self.clients);

        match self.socket.accept().await {
            Ok((stream, _addr)) => {
                let client_id = rand::random();
                let mut clients = clients.write().await;
                clients.insert(client_id, stream);
            }
            Err(_) => todo!(),
        }
    }

    async fn handle_client(client_id: u16, clients: ClientsArc) {
        //
        //
        //
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
            let socket_path = Ipc::<()>::get_socket_path(name);
            assert_eq!(socket_path.as_os_str(), "/tmp/test.sock");
        });
    }

    #[test]
    fn socket_path() {
        let socket_path = Ipc::<()>::get_socket_path("test.sock");
        assert_eq!(socket_path.as_os_str(), "/run/coreipc/test.sock");
    }

    #[tokio::test]
    async fn server_creation() -> std::io::Result<()> {
        // this ensures cleanup of the socket after test
        // you can verify this with `ls /tmp/.tmp*` before and after
        let socket = tempfile::Builder::new().make(|e| UnixListener::bind(e))?;
        Ipc::<()>::from_socket(socket.into_file())?;

        Ok(())
    }

    #[tokio::test]
    async fn server_creation_std() -> std::io::Result<()> {
        use std::os::unix::net::UnixListener;

        let socket = tempfile::Builder::new().make(|e| UnixListener::bind(e))?;
        Ipc::<()>::from_socket(socket.into_file())?;

        Ok(())
    }
}
