use response::LoginResp;
use s2n_quic::Connection;
use serde::{Deserialize, Serialize};

use crate::{db::Db, error::*, wire};

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    UpgradeConn,
    Login { name: String },
}

impl Action {
    pub async fn handle(&self, mut connection: Connection, db: Db) {
        match self {
            Action::UpgradeConn => todo!(),
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

                let resp = LoginResp { address, netmask };
                let data = wire::serialise_stream(&mut send_stream, &resp).await;

                if let Err(error) = data {
                    error!("{error}");
                }
            }
        };
    }
}

// TODO: impliment this for a server handler too, so we have a consistency of API
#[trait_variant::make(Send)]
pub trait ServerApi {
    async fn login(&self, username: &str) -> Result<LoginResp>;
    async fn upgrade_conn(&self) -> Result<()>;
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
