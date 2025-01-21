use relay_server::client::BidirectionalStream;
use tokio::sync::mpsc::{self, error::SendError};
use tun::Configuration as TunConfig;

use crate::{daemon::DaemonEvent, error};

pub const DEFAULT_MTU: u16 = 1500;

#[derive(Debug)]
pub enum TunEvent {
    SetRemote(Option<BidirectionalStream>),
    Up(TunConfig),
    Down,
}

#[derive(Debug)]
pub struct TunController {
    config: TunConfig,
}

impl TunController {
    pub fn new() -> Self {
        let mut config = TunConfig::default();

        config.tun_name("lanshare0").mtu(DEFAULT_MTU).up();

        TunController { config }
    }

    #[instrument(skip(self, rx, tun_tx))]
    pub async fn listen(
        &mut self,
        mut rx: mpsc::Receiver<DaemonEvent>,
        mut tun_tx: mpsc::Sender<TunEvent>,
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
    async fn handle_event(&mut self, event: DaemonEvent, tun_tx: &mut mpsc::Sender<TunEvent>) {
        trace!("TunController recieved event");
        match event {
            DaemonEvent::RemoteAdd { bi } => {
                handle_send_res(tun_tx.send(TunEvent::SetRemote(Some(bi))).await);
            }
            DaemonEvent::RemoteDel => {
                handle_send_res(tun_tx.send(TunEvent::SetRemote(None)).await);
            }
            DaemonEvent::Up { address, netmask } => {
                let mut config = self.config.clone();
                config.address(address).netmask(netmask);
                handle_send_res(tun_tx.send(TunEvent::Up(config)).await);
            }
            DaemonEvent::Down => {
                handle_send_res(tun_tx.send(TunEvent::Down).await);
            }
        }

        trace!("TunController event handeled");
    }
}

fn handle_send_res<T: std::fmt::Debug>(res: Result<(), SendError<T>>) {
    if let Err(error) = res {
        error!("no active recievers to recieve {:?}", error.0);
    }
}
