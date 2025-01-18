use std::net::Ipv4Addr;

use rusqlite::params;

use crate::{
    action::{response::*, ServerApi},
    db::Db,
    error::*,
};

impl ServerApi for Db {
    #[instrument(skip(self))]
    async fn login(&self, username: &str) -> Result<LoginResp> {
        let ip = new_ip();

        let db = self.db_conn.lock().await;

        let mut stmt = db.prepare("insert into users (username, ip) values (?1, ?2)")?;
        let rows_changed = stmt.execute(params![username, ip])?;

        if rows_changed != 1 {
            warn!("expected 1 but changed {rows_changed} rows");
        }

        Ok(LoginResp {
            address: ip.into(),
            netmask: Ipv4Addr::new(255, 0, 0, 0),
        })
    }

    async fn upgrade_conn(&self) -> Result<()> {
        todo!()
    }
}

fn new_ip() -> u32 {
    use rand::Rng as _;

    let mut rng = rand::thread_rng();

    // The first octet is fixed as 25. The rest are random.
    let first_octet = 25u32 << 24; // Shift 25 to the most significant byte.
    let second_octet = (rng.gen_range(0..=255) as u32) << 16;
    let third_octet = (rng.gen_range(0..=255) as u32) << 8;
    let fourth_octet = rng.gen_range(0..=255) as u32;

    // Combine the octets into a single u32.
    first_octet | second_octet | third_octet | fourth_octet
}
