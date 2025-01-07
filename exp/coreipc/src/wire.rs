//! serialising connection data to go over unix sockets

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientHello {
    pub socket_path: PathBuf,
}
