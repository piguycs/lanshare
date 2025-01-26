use std::sync::LazyLock;
use std::{collections::HashMap, net::Ipv4Addr, sync::Arc};

use bincode::{Decode, Encode};
use quic_abst::handler::Handler;
use quic_abst::reexports::quinn;
use tokio::sync::{Mutex, RwLock};

static TABLE: LazyLock<RwLock<HashMap<String, Ipv4Addr>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub struct VpnHandler {
    live_connections: Arc<Mutex<HashMap<Ipv4Addr, quinn::Connection>>>,
}

#[derive(Debug, Encode, Decode)]
pub enum HandlerInput {
    Login { username: String, password: String },
    Activate { token: String },
}

#[derive(Debug, Encode, Decode)]
pub enum HandlerOutput {
    LoginResp { ip: Ipv4Addr, token: String },
    SimpleAck { ack: bool },
}

impl Handler for VpnHandler {
    type In = HandlerInput;
    type Out = Result<HandlerOutput, ()>;

    async fn handle(&self, input: Self::In) -> Self::Out {
        use HandlerOutput::*;

        match input {
            HandlerInput::Login { username, password } => {
                let mut table = TABLE.write().await;

                if table.len() >= 255 {
                    return Err(());
                }

                let ip = Ipv4Addr::new(25, 0, 0, table.len() as u8);
                let token = format!("{username}{password}");

                table.insert(token.clone(), ip);

                Ok(LoginResp { ip, token })
            }
            HandlerInput::Activate { token } => {
                let table = TABLE.write().await;

                let _ip = match table.get(&token) {
                    Some(&value) => value,
                    None => return Err(()),
                };

                // TODO connection stuff
                Ok(SimpleAck { ack: true })
            }
        }
    }
}

impl VpnHandler {
    pub fn new() -> Self {
        Self {
            live_connections: Default::default(),
        }
    }

    pub async fn route_task(&self) {
        let mut users = None;

        loop {
            let table = {
                let table = TABLE.read().await;
                table.values().cloned().collect::<Vec<_>>()
            };

            if table.len() > 0 {
                users = Some(table);
            } else {
                users = None;
            }

            if let Some(_users) = users {
                //
            }
        }
    }
}
