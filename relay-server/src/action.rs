use std::net::Ipv4Addr;

use s2n_quic::Connection;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

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
                let data =
                    wire::serialise(&(Ipv4Addr::new(0, 0, 0, 0), (Ipv4Addr::new(0, 0, 0, 0))));

                let data = match data {
                    Ok(data) => data,
                    Err(error) => return error!("{error}"),
                };

                if let Err(error) = send_stream.write_all(&data).await {
                    error!("error when sending data: {error}")
                }
            }
        };
    }
}
