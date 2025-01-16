use rusqlite::Connection;
use tokio::sync::Mutex;

use std::{fmt::Debug, sync::Arc};

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
    pub fn try_new() -> crate::Result<Self> {
        let db = Connection::open_in_memory()?;

        Ok(Self {
            db_conn: Arc::new(Mutex::new(db)),
        })
    }

    pub async fn query(&self, query: &str) {
        todo!()
    }
}
