//! LANShare Client application

use tokio::{io::AsyncWriteExt, net::UnixStream};

#[macro_use]
extern crate tracing;

// static CERT: &str = include_str!("../../../certs/dev.crt");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("starting client");

    let mut sock = UnixStream::connect("lanshare.sock").await?;

    sock.write_all(b"hello world").await?;

    Ok(())
}
