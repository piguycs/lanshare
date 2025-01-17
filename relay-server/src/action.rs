use s2n_quic::Connection;
use serde::{Deserialize, Serialize};

use crate::{db::Db, error::*, wire};

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    Login { name: String },
}

impl Action {
    pub async fn handle(&self, mut connection: Connection, db: Db) {
        match self {
            Action::Login { name } => {
                info!(?name);
                let res = connection.open_send_stream().await.map_err(QuicError::from);

                let mut send_stream = match res {
                    Ok(value) => value,
                    Err(error) => return error!("could not open send stream: {error}"),
                };

                let (address, netmask) = match db.new_user_ip(name).await {
                    Ok(value) => value,
                    Err(error) => return error!(?error, "could not acquire address: {error}"),
                };
                info!("user {name} has been assigned address {address} and netmask {netmask}");

                let data = wire::serialise_stream(&mut send_stream, &(address, netmask)).await;

                if let Err(error) = data {
                    error!("{error}");
                }
            }
        };
    }
}
