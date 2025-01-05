//! Client for establishing a bi-directional connection to the server

use rand::Rng;
use tokio::io::AsyncWriteExt;
use tokio::net::{UnixListener, UnixStream};

use std::path::Path;
use std::{env, path::PathBuf};
use std::{format as f, io};

use crate::IntoSocket;

/// establish a bi-directional stream with the server
pub struct Client {
    // this is used to recieve connections from the server
    socket: UnixListener,
}

impl Client {
    pub fn create_client(name: &str) -> io::Result<Self> {
        let base_dir = get_runtime_dir();

        // we randomise the name, in case the user wants multiple clients running
        let rand_id: u32 = rand::thread_rng().gen_range(0..=0xFFFFFF);
        let sock_dir = base_dir.join(f!("{name}-{rand_id:06X}"));

        let socket = UnixListener::bind(sock_dir)?;

        Ok(Self { socket })
    }

    pub fn from_socket<S: IntoSocket>(client_socket: S) -> io::Result<Self> {
        let socket = client_socket.into_socket()?;

        Ok(Self { socket })
    }

    pub async fn connect<P: AsRef<Path>>(self, path: P) -> ConnectedClient {
        let server = UnixStream::connect(path).await.unwrap();

        ConnectedClient {
            server,
            recv_socket: self.socket,
        }
    }
}

pub struct ConnectedClient {
    server: UnixStream,
    recv_socket: UnixListener,
}

impl ConnectedClient {
    pub async fn send(&mut self, data: &[u8]) -> io::Result<()> {
        self.server.write_all(data).await?;

        Ok(())
    }

    // TODO: I think I will be using a 1:1 channel for this
    pub async fn get_cb<F: Fn(&[u8])>(&self, _cb: F) -> io::Result<()> {
        let _ = self.recv_socket.accept().await?;
        todo!()
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

    use super::*;

    #[test]
    fn runtime_dir() {
        let uid = unsafe { libc::getuid() };

        temp_env::with_var("XDG_RUNTIME_DIR", None::<String>, || {
            assert_eq!(get_runtime_dir(), PathBuf::from(f!("/run/{uid}")))
        });
    }

    #[tokio::test]
    async fn create_client_std() -> io::Result<()> {
        use std::os::unix::net::UnixListener;

        let socket = tempfile::Builder::new().make(|e| UnixListener::bind(e))?;

        Client::from_socket(socket.into_file())?;

        Ok(())
    }

    #[tokio::test]
    async fn create_client() -> io::Result<()> {
        let socket = tempfile::Builder::new().make(|e| UnixListener::bind(e))?;

        Client::from_socket(socket.into_file())?;

        Ok(())
    }

    #[tokio::test]
    async fn send_data() -> io::Result<()> {
        use std::time::Duration;
        use tokio::time::sleep;

        let server_socket = tempfile::Builder::new().make(|e| UnixListener::bind(e))?;

        let socket = tempfile::Builder::new().make(|e| UnixListener::bind(e))?;
        let client = Client::from_socket(socket.into_file())?;

        let path = server_socket.path();
        let mut client = client.connect(path).await;

        let test_data = b"hello world";

        client.send(test_data).await?;

        tokio::select! {
            Ok((mut stream, _)) = server_socket.as_file().accept() => {
                let mut buf = vec![];
                let _ = stream.read_buf(&mut buf).await.unwrap();

                assert_eq!(buf, test_data);
            }

            // 5 seconds should be more than enough
            _ = sleep(Duration::from_secs(5)) => panic!("timed out"),

            else => panic!("could not accept connection")
        };

        Ok(())
    }

    #[tokio::test]
    async fn recv_data() -> io::Result<()> {
        todo!()
    }
}
