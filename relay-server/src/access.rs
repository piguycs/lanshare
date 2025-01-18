use std::net::Ipv4Addr;

use rand::Rng as _;
use rusqlite::{params, ErrorCode};

use crate::{
    action::{response::*, ServerApi},
    db::Db,
    error::*,
};

impl ServerApi for Db {
    #[instrument(skip(self))]
    async fn login(&self, username: &str) -> Result<LoginResp> {
        let ip_address = new_ip();
        let token = gen_token::<16>();

        let db = self.db_conn.lock().await;

        let mut stmt = db.prepare("insert into users (username, ip, token) values (?1, ?2, ?3)")?;
        let res = stmt.execute(params![username, ip_address, token]);

        if let Err(error) = &res
            && error.sqlite_error_code() == Some(ErrorCode::ConstraintViolation)
        {
            warn!("user already exists");
            return Err(Error::UserAlreadyExists);
        }

        let rows_changed = res?;

        if rows_changed != 1 {
            warn!("expected 1 but changed {rows_changed} rows");
        }

        Ok(LoginResp {
            token,
            address: ip_address.into(),
            netmask: Ipv4Addr::new(255, 0, 0, 0),
        })
    }

    #[instrument(skip(self))]
    async fn upgrade_conn(&self, token: &str) -> Result<()> {
        todo!()
    }
}

//{{{ random generators
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

fn gen_token<const N: usize>() -> String {
    let mut rng = rand::thread_rng();
    let mut hex_string = String::new();

    for _ in 0..N {
        let random_byte: u8 = rng.gen_range(0..16);
        hex_string.push_str(&format!("{:x}", random_byte));
    }

    hex_string
}
//}}}
