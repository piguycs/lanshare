use rusqlite::Connection;
use tokio::sync::Mutex;

use std::{fmt::Debug, sync::Arc};

use crate::error::*;

#[derive(Clone)]
pub struct Db {
    db_conn: Arc<Mutex<Connection>>,
}

impl Debug for Db {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(in memory sqlite db)")
    }
}

impl Db {
    pub async fn try_new() -> Result<Self> {
        let db = Connection::open_in_memory()?;

        let db = Self {
            db_conn: Arc::new(Mutex::new(db)),
        };

        Ok(db)
    }

    pub async fn load_schema(&self, schema: &str) -> Result {
        let db = self.db_conn.lock().await;
        let mut stmt = db.prepare(schema).map_err(Error::SchemaError)?;
        stmt.execute([]).map_err(Error::SchemaError)?;

        Ok(())
    }
}
