use std::net::Ipv4Addr;

use s2n_quic::Connection;
use serde::{Deserialize, Serialize};

use crate::{db::Db, wire};

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    Login { name: String },
}

impl Action {
    pub async fn handle(&self, mut connection: Connection, _db: Db) {
        match self {
            Action::Login { name } => {
                info!(?name);
                let mut send_stream = connection.open_send_stream().await.unwrap();

                let data = wire::serialise_stream(
                    &mut send_stream,
                    &(Ipv4Addr::new(0, 0, 0, 0), (Ipv4Addr::new(0, 0, 0, 0))),
                )
                .await;

                if let Err(error) = data {
                    error!("{error}");
                }
            }
        };
    }
}
