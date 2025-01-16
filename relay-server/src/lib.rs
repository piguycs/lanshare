#[macro_use]
extern crate tracing;

pub mod client;
mod db;
pub mod error;
mod types;

use std::sync::LazyLock;

use bincode::Options;
use s2n_quic::{Connection, Server as QuicServer};
use tokio_util::io::SyncIoBridge;

use error::*;
use types::Action;

const SOCKET_ADDR: &str = "0.0.0.0:4433";

static CERT: &str = include_str!("../../certs/cert.pem");
static KEY: &str = include_str!("../../certs/key.pem");

const BINCODE_BYTE_LIMIT: u64 = 16 * 1024;
type BincodeConfig =
    bincode::config::WithOtherLimit<bincode::DefaultOptions, bincode::config::Bounded>;

static BINCODE: LazyLock<BincodeConfig> =
    LazyLock::new(|| bincode::DefaultOptions::new().with_limit(BINCODE_BYTE_LIMIT));

pub struct Server {
    db: db::Db,
    server: QuicServer,
}

impl Server {
    pub fn try_new() -> Result<Self> {
        let db = db::Db::get_db();

        let server = QuicServer::builder()
            .with_io(SOCKET_ADDR)
            .map_err(error::QuicError::from)?
            .with_tls((CERT, KEY))
            .expect("quic tls error: infailable")
            .start()
            .map_err(error::QuicError::from)?;

        Ok(Self { db, server })
    }

    #[instrument(skip(self))]
    pub async fn accept(&mut self) {
        info!("listening on {SOCKET_ADDR}");

        // docs on s2n_quic::server::Server::poll_accept say:
        // "Once None is returned, this function should not be called again"
        // or I would have ran this inside a loop {}
        while let Some(connection) = self.server.accept().await {
            tokio::spawn(handle_connection(connection, self.db.clone()));
        }

        debug!("quic server has been closed");
    }
}

// this function should ideally not "return" the error
// if it fails, we handle it here. propogating it upwards would be an error
#[instrument(skip(connection, db), fields(remote_addr = ?connection.remote_addr()))]
async fn handle_connection(mut connection: Connection, db: db::Db) {
    info!("Connection accepted from {:?}", connection.remote_addr());
    let recv_stream = match connection.accept_receive_stream().await {
        Ok(Some(value)) => {
            info!("WHAT THE FUCK BRO 1");
            value
        }
        // void returns, acts as an early exit
        Ok(None) => return debug!("stream was closed without an error"),
        Err(error) => return error!("{error}"),
    };
    info!("WHAT THE FUCK BRO 1");

    // TODO: BAD because blocking IO
    info!("reading action boobs");
    let action: Action = match BINCODE.deserialize_from(SyncIoBridge::new(recv_stream)) {
        Ok(value) => value,
        // void return, acts as an early exit
        Err(error) => return error!("{error}"),
    };

    info!("reading action boobs");

    match action {
        Action::Login { .. } => warn!("login is yet to be implimented"),
        Action::Test => {
            db.query("select 1 + 1").await;
        }
    };

    info!("connection ended");
}
