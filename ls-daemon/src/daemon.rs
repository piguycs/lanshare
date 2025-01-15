use relay_server::client::{Client, LoginS, Sender};
use tokio::sync::mpsc;

#[cfg(target_os = "linux")]
pub(super) use dbus::*;

#[derive(Debug, Clone, Copy)]
pub enum DaemonEvent {
    Up,
    Down,
}

pub trait Daemon {
    async fn login(&self, username: &str) -> usize;
    async fn int_up(&self) -> usize;
    async fn int_down(&self) -> usize;

    #[instrument(skip(tx))]
    async fn send_event(tx: &mpsc::Sender<DaemonEvent>, event: DaemonEvent) -> usize {
        if let Err(error) = tx.send(event).await {
            error!("{error}");
            return 1;
        }

        trace!("command to set the device state to {event:?} has been sent");
        0
    }
}

#[cfg(target_os = "linux")]
mod dbus {
    use tokio::sync::Mutex;
    use zbus::interface;

    use super::*;

    #[derive(Debug)]
    pub struct DbusDaemon {
        tx: mpsc::Sender<DaemonEvent>,
        relay_client: Mutex<Client>,
    }

    impl DbusDaemon {
        pub async fn new(tx: mpsc::Sender<DaemonEvent>) -> Self {
            let relay_client = Mutex::new(Client::create().await);
            Self { tx, relay_client }
        }
    }

    #[interface(name = "me.piguy.lanshare.daemon1")]
    impl Daemon for DbusDaemon {
        #[instrument(skip(self))]
        async fn login(&self, username: &str) -> usize {
            let mut client = self.relay_client.lock().await;

            client
                .send(LoginS {
                    name: username.to_string(),
                })
                .await;

            0
        }

        #[instrument(skip(self))]
        async fn int_up(&self) -> usize {
            Self::send_event(&self.tx, DaemonEvent::Up).await
        }

        #[instrument(skip(self))]
        async fn int_down(&self) -> usize {
            Self::send_event(&self.tx, DaemonEvent::Down).await
        }
    }
}
