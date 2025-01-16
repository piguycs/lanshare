#[macro_use]
extern crate tracing;

mod action;
pub mod client;
mod db;
pub mod error;

use std::{net::Ipv4Addr, sync::LazyLock};

use bincode::Options;
use s2n_quic::{Connection, Server as QuicServer};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{action::Action, error::*};

const SOCKET_ADDR: &str = "0.0.0.0:4433";
const PUBLIC_SOCKET_ADDR: &str = "127.0.0.1:4433";

static CERT: &str = include_str!("../../certs/cert.pem");
static KEY: &str = include_str!("../../certs/key.pem");

const BINCODE_BYTE_LIMIT: u64 = 16 * 1024;
type BincodeConfig =
    bincode::config::WithOtherLimit<bincode::DefaultOptions, bincode::config::Bounded>;

pub static BINCODE: LazyLock<BincodeConfig> =
    LazyLock::new(|| bincode::DefaultOptions::new().with_limit(BINCODE_BYTE_LIMIT));

pub struct Server {
    db: db::Db,
    server: QuicServer,
}

impl Server {
    pub async fn try_new() -> Result<Self> {
        let db = db::Db::try_new().await?;

        let schema = include_str!("../schemas/user-table.sql");
        db.load_schema(schema).await?;

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
    let mut recv_stream = match connection.accept_receive_stream().await {
        Ok(Some(value)) => value,
        // void returns, acts as an early exit
        Ok(None) => return debug!("stream was closed without an error"),
        Err(error) => return error!("{error}"),
    };

    // TODO: BAD because blocking IO
    let mut buf = vec![];
    let _len = recv_stream.read_buf(&mut buf).await.unwrap();
    let action = BINCODE.deserialize(&buf);
    debug!(?buf);
    drop(recv_stream);

    let action: Action = match action {
        Ok(value) => value,
        // void return, acts as an early exit
        Err(error) => return error!(?error, "{error}"),
    };

    info!("reading action boobs");

    action.handle(connection, db).await;

    info!("connection ended");
}
