use tokio::sync::{
    broadcast::{self, error::SendError},
    mpsc,
};
use tun::{Configuration as TunConfig, ToAddress};

use crate::{daemon::DaemonEvent, error};

pub const DEFAULT_MTU: u16 = 1500;

#[derive(Debug, Clone)]
pub enum TunEvent {
    Up(TunConfig),
    Down,
}

#[derive(Debug)]
pub struct TunController {
    config: TunConfig,
}

impl TunController {
    pub fn new<T: ToAddress>(address: T, netmask: T) -> Self {
        let mut config = TunConfig::default();

        config
            .tun_name("lanshare0")
            .address(address)
            .netmask(netmask)
            .mtu(DEFAULT_MTU)
            .up();

        TunController { config }
    }

    #[instrument(skip(self, rx, tun_tx))]
    pub async fn listen(
        &mut self,
        mut rx: mpsc::Receiver<DaemonEvent>,
        mut tun_tx: broadcast::Sender<TunEvent>,
    ) -> error::Result<()> {
        loop {
            if let Some(event) = rx.recv().await {
                self.handle_event(event, &mut tun_tx).await;
            } else {
                debug!("channel has been closed");
                return Ok(());
            }
        }
    }

    #[instrument(skip(self, tun_tx))]
    async fn handle_event(&mut self, event: DaemonEvent, tun_tx: &mut broadcast::Sender<TunEvent>) {
        trace!("TunController recieved event");
        match event {
            DaemonEvent::Up => {
                let config = self.config.clone();
                handle_send_res(tun_tx.send(TunEvent::Up(config)));
            }
            DaemonEvent::Down => {
                handle_send_res(tun_tx.send(TunEvent::Down));
            }
        }

        trace!("TunController event handeled");
    }
}

fn handle_send_res<T: std::fmt::Debug>(res: Result<usize, SendError<T>>) {
    if let Err(error) = res {
        error!("no active recievers to recieve {:?}", error.0);
    }
}
