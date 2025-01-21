pub mod handler;
pub mod response;

use std::net::Ipv4Addr;

use s2n_quic::{Connection, stream::BidirectionalStream};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{RoutingInfo, db::Db, error::*, wire};
use handler::ServerHandler;
use response::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    UpgradeConn { token: String },
    Login { name: String },
}

impl Action {
    #[instrument(skip(connection, db), fields(remote_addr = ?connection.remote_addr()))]
    pub async fn handle_action(
        self,
        connection: Connection,
        db: Db,
        tx: mpsc::Sender<RoutingInfo>,
    ) {
        match self {
            Action::UpgradeConn { token } => {
                let mut handler = ServerHandler { db, connection };
                let bi = match handler.upgrade(&token).await {
                    Ok(value) => value,
                    Err(error) => return error!("{error}"),
                };

                let (recv, send) = bi.split();
                let ri = RoutingInfo {
                    ip: Ipv4Addr::new(0, 0, 0, 0),
                    send,
                    recv,
                };
                if let Err(error) = tx.send(ri).await {
                    error!("could not send routing info: {error}");
                }
            }
            Action::Login { name } => {
                let data = match db.login(&name).await {
                    Ok(value) => value,
                    Err(error) => return error!("{error}"),
                };

                if let Err(error) = Self::send(connection, &data).await {
                    error!("error when sending handler response: {error}")
                }
            }
        }
    }

    #[instrument(skip(connection, data))]
    async fn send<T: Serialize>(mut connection: Connection, data: &T) -> Result<()> {
        let mut send_stream = connection
            .open_send_stream()
            .await
            .inspect_err(|error| error!(?error, "error when opening send stream"))
            .map_err(QuicError::from)?;

        wire::serialise_stream(&mut send_stream, data).await?;

        Ok(())
    }
}

#[trait_variant::make(Send)]
pub trait ServerApi {
    async fn login(&self, username: &str) -> Result<LoginResp>;
    async fn upgrade_conn(&self, token: &str) -> Result<BidirectionalStream>;
}
