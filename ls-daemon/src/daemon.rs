use std::net::Ipv4Addr;

use tokio::sync::mpsc;

use errors::*;
use relay_server::client::Client;

#[cfg(target_os = "linux")]
pub(super) use dbus::*;

pub const SERVER_ADDR: &str = "127.0.0.1:4433";

#[derive(Debug, Clone, Copy)]
pub enum DaemonEvent {
    Up {
        address: Ipv4Addr,
        netmask: Ipv4Addr,
    },
    Down,
}

pub trait Daemon {
    // this function is expected to modify the login state
    async fn login(&mut self, username: &str) -> usize;

    async fn int_up(&self) -> usize;
    async fn int_down(&self) -> usize;

    #[instrument(skip(tx))]
    async fn send_event(tx: &mpsc::Sender<DaemonEvent>, event: DaemonEvent) -> usize {
        if let Err(error) = tx.send(event).await {
            error!("{error}");
            return DAEMON_ERROR;
        }

        trace!("command to set the device state to {event:?} has been sent");
        0
    }
}

#[cfg(target_os = "linux")]
mod dbus {
    use std::{
        net::{Ipv4Addr, SocketAddr},
        str::FromStr,
    };

    use zbus::interface;

    use crate::error::Result;

    use super::*;

    #[derive(Debug)]
    pub struct DbusDaemon {
        tx: mpsc::Sender<DaemonEvent>,
        relay_client: Client,
        login_cfg: Option<(Ipv4Addr, Ipv4Addr)>,
    }

    impl DbusDaemon {
        pub async fn try_new(tx: mpsc::Sender<DaemonEvent>) -> Result<Self> {
            let server_addr = SocketAddr::from_str(SERVER_ADDR).expect("infailable");
            let relay_client = Client::try_new(server_addr).await?;
            Ok(Self {
                tx,
                relay_client,
                login_cfg: None,
            })
        }
    }

    #[interface(name = "me.piguy.lanshare.daemon1")]
    impl Daemon for DbusDaemon {
        #[instrument(skip(self))]
        async fn login(&mut self, username: &str) -> usize {
            let client = &self.relay_client;
            let login_cfg = match client.login(username).await {
                Ok(value) => value,
                Err(error) => {
                    error!("could not login user: {error}");
                    return LOGIN_INVALID;
                }
            };

            self.login_cfg = Some(login_cfg);

            0
        }

        #[instrument(skip(self))]
        async fn int_up(&self) -> usize {
            if let Some((address, netmask)) = self.login_cfg {
                Self::send_event(&self.tx, DaemonEvent::Up { address, netmask }).await
            } else {
                CLOSED_CHANNEL
            }
        }

        #[instrument(skip(self))]
        async fn int_down(&self) -> usize {
            Self::send_event(&self.tx, DaemonEvent::Down).await
        }
    }
}
