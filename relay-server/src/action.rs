use std::net::Ipv4Addr;

use bincode::Options;
use s2n_quic::Connection;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

use crate::{db::Db, BINCODE};

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    Login { name: String },
}

impl Action {
    pub async fn handle(&self, mut connection: Connection, db: Db) {
        match self {
            Action::Login { name } => {
                info!(?name);
                let mut send_stream = connection.open_send_stream().await.unwrap();
                let data = BINCODE
                    .serialize(&(Ipv4Addr::new(0, 0, 0, 0), (Ipv4Addr::new(0, 0, 0, 0))))
                    .unwrap();
                send_stream.write_all(&data).await.unwrap();
            }
        };
    }
}
