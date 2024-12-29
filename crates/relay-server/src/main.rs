//! Relay-Server for LANShare
//! W.I.P. server for connecting LANShare clients

#[macro_use]
extern crate tracing;

use s2n_quic::Server;

type BoxError = Box<dyn std::error::Error>;

const SOCKET_ADDR: &str = "0.0.0.0:4433";

static CERT: &str = include_str!("../../../certs/dev.crt");
static KEY: &str = include_str!("../../../certs/dev.key");

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    tracing_subscriber::fmt::init();

    let mut server = Server::builder()
        .with_io(SOCKET_ADDR)?
        .with_tls((CERT, KEY))?
        .start()?;

    trace!("accepting connections");
    while let Some(mut _connection) = server.accept().await {
        info!("hello world");
        tokio::spawn(async {});
        //
    }

    Ok(())
}
