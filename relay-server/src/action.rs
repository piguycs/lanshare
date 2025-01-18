use s2n_quic::Connection;
use serde::{Deserialize, Serialize};

use crate::{db::Db, error::*, wire};
use handler::*;
use response::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    UpgradeConn,
    Login { name: String },
}

impl ServerHandler {
    #[instrument(skip(connection, db), fields(remote_addr = ?connection.remote_addr()))]
    pub async fn handle_action(action: Action, connection: Connection, db: Db) {
        let handler = Self { db };

        match action {
            Action::UpgradeConn => todo!(),
            Action::Login { name } => {
                let data = match handler.login(&name).await {
                    Ok(value) => value,
                    Err(error) => return error!("{error}"),
                };

                if let Err(error) = Self::send(connection, &data).await {
                    error!("error when sending handler response: {error}")
                }
            }
        }
    }

    async fn send<T: Serialize>(mut connection: Connection, data: &T) -> Result<()> {
        let mut send_stream = connection
            .open_send_stream()
            .await
            .map_err(QuicError::from)?;

        wire::serialise_stream(&mut send_stream, data).await?;

        Ok(())
    }
}

#[trait_variant::make(Send)]
pub trait ServerApi {
    async fn login(&self, username: &str) -> Result<LoginResp>;
    async fn upgrade_conn(&self) -> Result<()>;
}

pub mod handler {
    use super::*;

    pub struct ServerHandler {
        pub(super) db: Db,
    }

    impl ServerApi for ServerHandler {
        #[instrument(skip(self))]
        async fn login(&self, username: &str) -> Result<LoginResp> {
            let db = &self.db;
            let (address, netmask) = db.new_user_ip(username).await?;
            info!("user {username} has been assigned address {address} and netmask {netmask}");

            let resp = LoginResp { address, netmask };

            Ok(resp)
        }

        #[instrument(skip(self))]
        async fn upgrade_conn(&self) -> Result<()> {
            todo!()
        }
    }
}

pub mod response {
    use super::*;

    use std::net::Ipv4Addr;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct LoginResp {
        pub address: Ipv4Addr,
        pub netmask: Ipv4Addr,
    }
}
