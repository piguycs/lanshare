use tokio::sync::Mutex;

use std::{fmt::Debug, marker::PhantomData, sync::Arc};

#[derive(Clone)]
pub struct MemSqlite;

#[derive(Clone)]
pub struct Db<T = MemSqlite> {
    db_conn: Arc<Mutex<()>>,
    db_type: PhantomData<T>,
}

impl Debug for Db<MemSqlite> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(in memory sqlite db)")
    }
}

impl Db<MemSqlite> {
    pub fn get_db() -> Self {
        Self {
            db_conn: Arc::default(),
            db_type: PhantomData,
        }
    }

    pub async fn query(&self, query: &str) {
        todo!()
    }
}
