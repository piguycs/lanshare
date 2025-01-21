#![feature(let_chains)]

#[macro_use]
extern crate tracing;

pub mod access;
mod action;
pub mod client;
mod db;
pub mod error;
mod wire;

use std::collections::HashMap;
use std::net::Ipv4Addr;

use db::Db;
use s2n_quic::stream::{ReceiveStream, SendStream};
use s2n_quic::{Connection, Server as QuicServer};
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::RwLock;

use crate::{action::Action, error::*};

const SOCKET_ADDR: &str = "0.0.0.0:4433";

static CERT: &str = include_str!("../../certs/cert.pem");
static KEY: &str = include_str!("../../certs/key.pem");

pub struct RoutingInfo {
    ip: Ipv4Addr,
    recv: ReceiveStream,
    send: SendStream,
}

pub struct Server {
    db: Db,
    server: QuicServer,
}

impl Server {
    pub async fn try_new() -> Result<Self> {
        let db = Db::try_new().await?;

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

        let (tx, rx) = mpsc::channel(16);
        tokio::spawn(async { handle_routing(rx).await });

        // docs on s2n_quic::server::Server::poll_accept say:
        // "Once None is returned, this function should not be called again"
        // or I would have ran this inside a loop {}
        while let Some(connection) = self.server.accept().await {
            tokio::spawn(handle_connection(connection, self.db.clone(), tx.clone()));
        }

        debug!("quic server has been closed");
    }
}

// this function should ideally not "return" the error
// if it fails, we handle it here. propogating it upwards would be an error
#[instrument(skip(connection, db, tx), fields(remote_addr = ?connection.remote_addr()))]
async fn handle_connection(mut connection: Connection, db: Db, tx: Sender<RoutingInfo>) {
    info!("Connection accepted from {:?}", connection.remote_addr());
    let mut recv_stream = match connection.accept_receive_stream().await {
        Ok(Some(value)) => value,
        // void returns, acts as an early exit
        Ok(None) => return debug!("stream was closed without an error"),
        Err(error) => return error!("{error}"),
    };

    let action = wire::deserialise_stream(&mut recv_stream).await;
    drop(recv_stream);

    let action: Action = match action {
        Ok(value) => value,
        // void return, acts as an early exit
        Err(error) => return error!(?error, "{error}"),
    };

    action.handle_action(connection, db, tx).await;

    info!("connection ended");
}

#[instrument(skip(recv))]
async fn parsepkt(mut recv: ReceiveStream) {
    let mut buf = [0; 4096];
    while let Ok(amount) = recv.read(&mut buf).await {
        match etherparse::Ipv4Slice::from_slice(&buf[..amount]) {
            Ok(packet) => debug!(?packet, "ipv4 packet"),
            Err(error) => error!(?error, "could not parse packet: {error}"),
        }
    }
}

#[instrument(skip(rx))]
async fn handle_routing(mut rx: mpsc::Receiver<RoutingInfo>) {
    let route_table = RwLock::new(HashMap::new());

    while let Some(RoutingInfo { ip, send, recv }) = rx.recv().await {
        let mut route_table = route_table.write().await;
        route_table.insert(ip, send);
        drop(route_table);

        tokio::spawn(parsepkt(recv));
    }
}
