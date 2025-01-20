use super::*;

use std::net::Ipv4Addr;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResp {
    pub token: String,
    pub address: Ipv4Addr,
    pub netmask: Ipv4Addr,
}
