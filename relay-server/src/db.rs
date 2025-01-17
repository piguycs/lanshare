use std::{fmt::Debug, net::Ipv4Addr, sync::Arc};

use rand::Rng;
use rusqlite::Connection;
use tokio::sync::Mutex;

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

    pub async fn new_user_ip(&self, username: &str) -> Result<(Ipv4Addr, Ipv4Addr)> {
        let ip = new_ip();

        let db = self.db_conn.lock().await;
        let mut stmt = db
            .prepare("insert into users (username, ip) values (?1, ?2)")
            .map_err(Error::SqlError)?;
        let rows_changed = stmt
            .execute(rusqlite::params![username, ip])
            .map_err(Error::SqlError)?;

        if rows_changed != 1 {
            warn!(
                ?rows_changed,
                "incorrect number of rows changed, expected 1"
            );
        }

        Ok((Ipv4Addr::from_bits(ip), Ipv4Addr::new(255, 0, 0, 0)))
    }
}

fn new_ip() -> u32 {
    let mut rng = rand::thread_rng();

    // The first octet is fixed as 25. The rest are random.
    let first_octet = 25u32 << 24; // Shift 25 to the most significant byte.
    let second_octet = (rng.gen_range(0..=255) as u32) << 16;
    let third_octet = (rng.gen_range(0..=255) as u32) << 8;
    let fourth_octet = rng.gen_range(0..=255) as u32;

    // Combine the octets into a single u32.
    first_octet | second_octet | third_octet | fourth_octet
}
