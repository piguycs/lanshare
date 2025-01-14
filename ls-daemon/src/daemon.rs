#[cfg(target_os = "linux")]
pub use dbus::*;

pub trait Daemon {
    async fn login(&self) -> usize;
    async fn int_up(&self) -> usize;
    async fn int_down(&self) -> usize;
}

#[cfg(target_os = "linux")]
mod dbus {
    use tokio::sync::mpsc;
    use zbus::interface;

    use super::*;

    #[derive(Debug, Clone, Copy)]
    pub enum DaemonEvent {
        Up,
        Down,
    }

    #[derive(Debug)]
    pub struct DbusDaemon {
        tx: mpsc::Sender<DaemonEvent>,
    }

    impl DbusDaemon {
        pub fn new(tx: mpsc::Sender<DaemonEvent>) -> Self {
            Self { tx }
        }

        #[instrument(skip(tx))]
        async fn send_event(tx: mpsc::Sender<DaemonEvent>, event: DaemonEvent) -> usize {
            if let Err(error) = tx.send(event).await {
                error!("{error}");
                return 1;
            }

            trace!("command to set the device state to {event:?} has been sent");
            0
        }
    }

    #[interface(name = "me.piguy.lanshare.daemon1")]
    impl Daemon for DbusDaemon {
        #[instrument(skip(self))]
        async fn login(&self) -> usize {
            todo!()
        }

        #[instrument(skip(self))]
        async fn int_up(&self) -> usize {
            let tx = self.tx.clone();
            Self::send_event(tx, DaemonEvent::Up).await
        }

        #[instrument(skip(self))]
        async fn int_down(&self) -> usize {
            let tx = self.tx.clone();
            Self::send_event(tx, DaemonEvent::Down).await
        }
    }
}
