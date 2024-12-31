//! LANShare Client application

use std::net::SocketAddr;

use s2n_quic::{client::Connect, Client};

#[macro_use]
extern crate tracing;

static CERT: &str = include_str!("../../../certs/dev.crt");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let client = Client::builder()
        .with_tls(CERT)?
        .with_io("0.0.0.0:0")?
        .start()?;

    let addr: SocketAddr = "127.0.0.1:4433".parse()?;
    let connect = Connect::new(addr).with_server_name("localhost");
    let mut connection = client.connect(connect).await?;

    let stream = connection.open_bidirectional_stream().await?;
    let (mut receive_stream, mut send_stream) = stream.split();

    // TODO: this is the part where we forward packets to our TUN device
    tokio::spawn(async move {
        let mut stdout = tokio::io::stdout();
        if let Err(error) = tokio::io::copy(&mut receive_stream, &mut stdout).await {
            error!(?receive_stream, "could not copy stream to stdout: {error}");
        }
    });

    let mut stdin = tokio::io::stdin();
    tokio::io::copy(&mut stdin, &mut send_stream).await?;

    Ok(())
}
