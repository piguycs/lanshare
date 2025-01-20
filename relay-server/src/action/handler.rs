use s2n_quic::stream::BidirectionalStream;
use s2n_quic::Connection;

use crate::db::Db;
use crate::error::*;

pub struct ServerHandler {
    pub(super) db: Db,
    pub(super) connection: Connection,
}

impl ServerHandler {
    pub async fn upgrade(&mut self, token: &str) -> Result<BidirectionalStream> {
        let db = self.db.db_conn.lock().await;

        let name: String = db
            .query_row(
                "select username from users where token = ?1",
                [token],
                |e| {
                    debug!(?e);
                    e.get(0)
                },
            )
            .inspect_err(|e| error!(?e))?;
        drop(db);

        info!("upgrading connection for {name}");

        let bi = self
            .connection
            .open_bidirectional_stream()
            .await
            .map_err(QuicError::from)?;

        Ok(bi)
    }
}
