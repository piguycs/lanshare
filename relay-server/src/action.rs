use s2n_quic::Connection;
use serde::{Deserialize, Serialize};

use crate::{db::Db, error::*, wire};
use response::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    UpgradeConn,
    Login { name: String },
}

impl Action {
    #[instrument(skip(connection, db), fields(remote_addr = ?connection.remote_addr()))]
    pub async fn handle_action(self, mut connection: Connection, db: Db) {
        match self {
            Action::UpgradeConn => {
                let bi = connection.open_bidirectional_stream().await.unwrap();
                debug!(?bi);

                if let Err(error) = connection.keep_alive(true) {
                    error!("Connection::keep_alive failed: {error}");
                }

                let (mut recv, mut send) = bi.split();

                tokio::spawn(async move {
                    print!("[CLIENT] ");
                    let mut stdout = tokio::io::stdout();
                    let _ = tokio::io::copy(&mut recv, &mut stdout).await;
                });

                let mut stdin = tokio::io::stdin();
                tokio::io::copy(&mut stdin, &mut send).await.unwrap();
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
    async fn upgrade_conn(&self, token: &str) -> Result<()>;
}

pub mod response {
    use super::*;

    use std::net::Ipv4Addr;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct LoginResp {
        pub token: String,
        pub address: Ipv4Addr,
        pub netmask: Ipv4Addr,
    }
}
