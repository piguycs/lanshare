use tokio::sync::{broadcast, mpsc};
use tun::Configuration as TunConfig;

use crate::{daemon::DaemonEvent, error};

pub const DEFAULT_MTU: u16 = 1500;

#[derive(Debug)]
pub struct TunController {
    config: tun::Configuration,
}

impl TunController {
    pub fn new() -> Self {
        let mut config = TunConfig::default();
        config
            .address((25, 0, 0, 2))
            .netmask((255, 0, 0, 0))
            .mtu(DEFAULT_MTU)
            .up();

        TunController { config }
    }

    #[instrument(skip(self, rx, tun_tx))]
    pub async fn listen(
        &mut self,
        mut rx: mpsc::Receiver<DaemonEvent>,
        mut tun_tx: broadcast::Sender<Option<TunConfig>>,
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
    async fn handle_event(
        &mut self,
        event: DaemonEvent,
        tun_tx: &mut broadcast::Sender<Option<TunConfig>>,
    ) {
        trace!("TunController recieved event");
        match event {
            DaemonEvent::Up => {
                tun_tx.send(Some(self.config.clone())).unwrap();
            }
            DaemonEvent::Down => {
                tun_tx.send(None).unwrap();
            }
        }

        trace!("TunController event handeled");
    }
}
