//! serialising connection data to go over unix sockets

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ClientHello {
    pub socket_path: PathBuf,
}
