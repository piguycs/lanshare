//! Client for establishing a bi-directional connection to the server
//!
//! ## Example
//! ```rust
//! let client = Client::create_client("ipc.client");
//! let client = client.connnect("ipc.sock"); // the name of your server
//!
//! // TODO: get_cb and send documentation
//! ```

use rand::Rng;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

use std::format as f;
use std::path::Path;
use std::{env, path::PathBuf};

use crate::error::*;
use crate::wire::ClientHello;
use crate::IntoSocket;

/// establish a bi-directional stream with the server
pub struct Client {
    // this is used to recieve connections from the server
    socket: UnixListener,
    socket_path: PathBuf,
}

impl Client {
    pub fn create_client(name: &str) -> Result<Self> {
        let base_dir = get_runtime_dir();

        // we randomise the name, in case the user wants multiple clients running
        let rand_id: u32 = rand::thread_rng().gen_range(0..=0xFFFFFF);
        let socket_path = base_dir.join(f!("{name}-{rand_id:06X}"));

        let socket = UnixListener::bind(&socket_path)?;

        Ok(Self {
            socket,
            socket_path,
        })
    }

    pub fn from_socket<S: IntoSocket>(client_socket: S) -> Result<Self> {
        let socket = client_socket.into_socket()?;

        let binding = socket.local_addr()?;
        let path = binding.as_pathname().ok_or(Error::UnnamedSocket)?;

        Ok(Self {
            socket,
            socket_path: path.to_path_buf(),
        })
    }

    pub async fn connect<P: AsRef<Path>>(self, path: P) -> Result<ConnectedClient> {
        let mut server = UnixStream::connect(path).await?;

        let hello = bincode::serialize(&ClientHello {
            socket_path: self.socket_path,
        })?;

        assert!(hello.len() < u16::MAX as usize);

        server.write_u16(hello.len() as u16).await?;
        server.write_all(&hello).await?;

        Ok(ConnectedClient {
            server,
            recv_socket: self.socket,
        })
    }
}

pub struct ConnectedClient {
    server: UnixStream,
    recv_socket: UnixListener,
}

impl ConnectedClient {
    pub async fn send(&mut self, data: &[u8]) -> Result<()> {
        self.server.write_all(data).await?;

        Ok(())
    }

    // TODO: I think I will be using a 1:1 channel for this
    pub async fn get_cb(&self) -> Result<()> {
        loop {
            if let Ok((mut stream, _)) = self.recv_socket.accept().await {
                tokio::spawn(async move {
                    let mut buf = vec![];
                    let len = stream.read_buf(&mut buf).await.unwrap();
                    debug!("{:?}", &buf[..len]);
                });
            }
        }
    }
}

fn get_runtime_dir() -> PathBuf {
    let dir = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
        let uid = unsafe { libc::getuid() };

        format!("/run/{uid}")
    });

    dir.into()
}

#[cfg(test)]
mod unit_test {
    use tokio::io::AsyncReadExt;

    type TempSocket = tempfile::NamedTempFile<UnixListener>;

    use super::*;

    #[test]
    fn runtime_dir() {
        let uid = unsafe { libc::getuid() };

        temp_env::with_var("XDG_RUNTIME_DIR", None::<String>, || {
            assert_eq!(get_runtime_dir(), PathBuf::from(f!("/run/{uid}")))
        });
    }

    #[tokio::test]
    async fn create_client_std() -> Result<()> {
        use std::os::unix::net::UnixListener;

        let socket = tempfile::Builder::new().make(|e| UnixListener::bind(e))?;

        Client::from_socket(socket.into_file())?;

        Ok(())
    }

    #[tokio::test]
    async fn create_client() -> Result<()> {
        let socket = tempfile::Builder::new().make(|e| UnixListener::bind(e))?;

        Client::from_socket(socket.into_file())?;

        Ok(())
    }

    #[rstest::fixture]
    fn server_socket() -> TempSocket {
        tempfile::Builder::new()
            .make(|e| UnixListener::bind(e))
            .expect("could not create a server socket")
    }

    #[rstest::rstest]
    #[case(b"hello_world, this is a simple test")]
    #[case(&[0xFF, 0x00, 0x1C, 0x7E, 0x03, 0x42, 0x88, 0x10])]
    #[case(b"dog")]
    #[tokio::test]
    async fn send_data(server_socket: TempSocket, #[case] test_data: &[u8]) -> Result<()> {
        let socket = tempfile::Builder::new().make(|e| UnixListener::bind(e))?;
        let client = Client::from_socket(socket.into_file())?;

        let path = server_socket.path();
        let mut client = client.connect(path).await?;

        client.server.write_all(test_data).await?;

        match server_socket.as_file().accept().await {
            Ok((mut stream, _)) => {
                let mut buf = vec![];
                let _ = stream.read_buf(&mut buf).await.unwrap();

                let data_start = buf.len() - test_data.len();

                assert_eq!(&buf[data_start..], test_data);
            }
            Err(error) => {
                dbg!(error);
                panic!("could not accept connection");
            }
        }

        Ok(())
    }

    #[rstest::rstest]
    #[tokio::test]
    async fn hello(server_socket: TempSocket) -> Result<()> {
        let socket = server_socket.as_file();

        let _ = socket.local_addr()?.as_pathname().unwrap();

        Ok(())
    }
}
