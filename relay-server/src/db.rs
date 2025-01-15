use tokio::sync::Mutex;

use std::{fmt::Debug, marker::PhantomData, sync::Arc};

#[derive(Clone)]
pub struct MemSqlite;

#[derive(Clone)]
pub struct Db<T = MemSqlite> {
    db_conn: Arc<Mutex<sqlite::Connection>>,
    db_type: PhantomData<T>,
}

impl Debug for Db<MemSqlite> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(in memory sqlite db)")
    }
}

impl Db<MemSqlite> {
    pub fn get_db() -> Self {
        let conn = sqlite::open(":memory:").unwrap();

        Self {
            db_conn: Arc::new(Mutex::new(conn)),
            db_type: PhantomData,
        }
    }

    pub async fn query(&self, query: &str) {
        let db_conn = self.db_conn.lock().await;

        db_conn
            .iterate(query, |pairs| {
                for &(name, value) in pairs {
                    println!("{name:?}, {value:?}");
                }

                true
            })
            .unwrap();
    }
}
