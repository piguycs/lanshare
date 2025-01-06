//! Relay-Server for LANShare
//! This is the binary part of the relay server. There is also a library with all the common types

#[macro_use]
extern crate tracing;

use s2n_quic::{stream::BidirectionalStream, Connection, Server};

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
    while let Some(connection) = server.accept().await {
        tokio::spawn(handle_connection(connection));
    }

    Ok(())
}

async fn handle_connection(mut connection: Connection) {
    info!("Connection accepted from {:?}", connection.remote_addr());

    while let Ok(Some(stream)) = connection.accept_bidirectional_stream().await {
        tokio::spawn(handle_stream(stream));
    }
}

async fn handle_stream(mut stream: BidirectionalStream) {
    info!("Stream opened from {:?}", stream.connection().remote_addr());

    while let Ok(Some(data)) = stream.receive().await {
        debug!("stream sent {:?}", String::from_utf8(data.to_vec()));

        stream
            .send("ACK".into())
            .await
            .expect("stream should be open");
    }
}
