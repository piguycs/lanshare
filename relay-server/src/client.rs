use s2n_quic::{
    client::Connect,
    stream::{ReceiveStream, SendStream},
};
use tokio::io::AsyncWriteExt;

use std::net::SocketAddr;

static CERT: &str = include_str!("../../certs/cert.pem");

pub struct LoginS {
    pub name: String,
}

pub struct LoginR;

#[derive(Debug)]
pub struct Client {
    recv: ReceiveStream,
    send: SendStream,
}

impl Client {
    pub async fn create() -> Self {
        warn!("this connection is insecure, as it uses a globally issued certificate");

        // TODO: accept cert from caller
        let client = s2n_quic::Client::builder()
            .with_tls(CERT)
            .unwrap()
            .with_io("0.0.0.0:0")
            .unwrap()
            .start()
            .unwrap();

        let addr: SocketAddr = "127.0.0.1:4433".parse().unwrap();
        let connect = Connect::new(addr).with_server_name("localhost");
        let mut connection = client.connect(connect).await.unwrap();

        let stream = connection.open_bidirectional_stream().await.unwrap();
        let (recv, send) = stream.split();

        Self { recv, send }
    }
}

#[allow(async_fn_in_trait)]
pub trait Sender<S, R> {
    async fn send(&mut self, s: S) -> R;
}

impl Sender<LoginS, LoginR> for Client {
    async fn send(&mut self, s: LoginS) -> LoginR {
        self.send.write_all(s.name.as_bytes()).await.unwrap();

        // TODO: todo
        LoginR
    }
}
