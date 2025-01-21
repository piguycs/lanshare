use std::net::Ipv4Addr;

use s2n_quic::stream::BidirectionalStream;
use s2n_quic::Connection;

use crate::db::Db;
use crate::error::*;

pub struct ServerHandler {
    pub(super) db: Db,
    pub(super) connection: Connection,
}

impl ServerHandler {
    pub async fn upgrade(&mut self, token: &str) -> Result<(Ipv4Addr, BidirectionalStream)> {
        let db = self.db.db_conn.lock().await;

        let ip_bits: u32 = db
            .query_row("select ip from users where token = ?1", [token], |e| {
                debug!(?e);
                e.get(0)
            })
            .inspect_err(|e| error!(?e))?;
        drop(db);

        let ip = Ipv4Addr::from_bits(ip_bits);

        let bi = self
            .connection
            .open_bidirectional_stream()
            .await
            .map_err(QuicError::from)?;

        Ok((ip, bi))
    }
}
