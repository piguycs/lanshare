//! Relay-Server for LANShare
//! This is the binary part of the relay server. There is also a library with all the common types

#[macro_use]
extern crate tracing;

use std::sync::Arc;

use relay_server::types::Actions;
use s2n_quic::{stream::BidirectionalStream, Connection, Server};
use tokio::sync::Mutex;

type BoxError = Box<dyn std::error::Error>;

const SOCKET_ADDR: &str = "0.0.0.0:4433";

static CERT: &str = include_str!("../../certs/cert.pem");
static KEY: &str = include_str!("../../certs/key.pem");

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    tracing_subscriber::fmt::init();

    let db_conn = relay_server::db::gen_mem_db();
    let db_conn = Arc::new(Mutex::new(db_conn));

    let mut server = Server::builder()
        .with_io(SOCKET_ADDR)?
        .with_tls((CERT, KEY))?
        .start()?;

    trace!("accepting connections");
    while let Some(connection) = server.accept().await {
        tokio::spawn(handle_connection(connection, db_conn.clone()));
    }

    Ok(())
}

async fn handle_connection(mut connection: Connection, db_conn: Arc<Mutex<sqlite::Connection>>) {
    info!("Connection accepted from {:?}", connection.remote_addr());

    while let Ok(Some(stream)) = connection.accept_bidirectional_stream().await {
        tokio::spawn(handle_stream(stream, db_conn.clone()));
    }
}

async fn handle_stream(mut stream: BidirectionalStream, db_conn: Arc<Mutex<sqlite::Connection>>) {
    info!("Stream opened from {:?}", stream.connection().remote_addr());

    while let Ok(Some(data)) = stream.receive().await {
        let actions: Actions = bincode::deserialize(&data).unwrap();
        debug!("stream sent {actions:?}");

        stream
            .send("ACK".into())
            .await
            .expect("stream should be open");
    }
}
