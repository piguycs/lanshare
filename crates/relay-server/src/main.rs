//! Relay-Server for LANShare
//! W.I.P. server for connecting LANShare clients

#[macro_use]
extern crate tracing;

use s2n_quic::Server;

type BoxError = Box<dyn std::error::Error>;

const SOCKET_ADDR: &str = "0.0.0.0:4433";

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    tracing_subscriber::fmt::init();

    let mut server = Server::builder().with_io(SOCKET_ADDR)?.start()?;

    trace!("accepting connections");
    while let Some(mut connection) = server.accept().await {
        info!("hello world");
        //
    }

    Ok(())
}
