#[macro_use]
extern crate tracing;

use rand::Rng;
use relay_server::{db::Db, types::Actions};
use s2n_quic::{stream::BidirectionalStream, Connection, Server};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

type BoxError = Box<dyn std::error::Error>;

const SOCKET_ADDR: &str = "0.0.0.0:4433";

static CERT: &str = include_str!("../../certs/cert.pem");
static KEY: &str = include_str!("../../certs/key.pem");

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    tracing_subscriber::fmt::init();

    let db_conn = Db::get_db();

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

async fn handle_connection(mut connection: Connection, db_conn: Db) {
    info!("Connection accepted from {:?}", connection.remote_addr());

    while let Ok(Some(stream)) = connection.accept_bidirectional_stream().await {
        tokio::spawn(handle_stream(stream, db_conn.clone()));
    }
}

async fn handle_stream(mut stream: BidirectionalStream, db_conn: Db) {
    info!("Stream opened from {:?}", stream.connection().remote_addr());

    loop {
        let len = stream.read_u32().await.unwrap();
        info!(?len);
        let mut buf = vec![0; len as usize];
        if let Ok(amount) = stream.read_exact(&mut buf).await {
            info!("got here");
            if let Ok(Actions::Login { name }) = bincode::deserialize::<Actions>(&buf[..amount]) {
                db_conn.query("select 1 + 1").await;
                info!("data: {name}");

                let rand_ip = ip();

                stream.write_u32(rand_ip).await.unwrap();
            } else {
                warn!("nope");
            }
        }
    }
}

fn ip() -> u32 {
    let mut rng = rand::thread_rng();

    // The first octet is fixed as 25. The rest are random.
    let first_octet = 25u32 << 24; // Shift 25 to the most significant byte.
    let second_octet = (rng.gen_range(0..=255) as u32) << 16;
    let third_octet = (rng.gen_range(0..=255) as u32) << 8;
    let fourth_octet = rng.gen_range(0..=255) as u32;

    // Combine the octets into a single u32.
    first_octet | second_octet | third_octet | fourth_octet
}
