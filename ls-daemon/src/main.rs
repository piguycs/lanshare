#![feature(never_type)]

#[macro_use]
extern crate tracing;

mod daemon;
mod error;
mod tun;

use std::io::Read;

use tokio::sync::{broadcast, mpsc};
use tun::TunEvent;
use zbus::connection;

use crate::{
    daemon::{DaemonEvent, DbusDaemon},
    tun::TunController,
};

#[tokio::main]
async fn main() -> error::Result {
    tracing_subscriber::fmt::init();
    info!("hello from daemon");

    // channel for recieving events from dbus
    let (tx, rx) = mpsc::channel::<DaemonEvent>(1);
    // channel for recieving tun device from TunController
    let (tun_tx, tun_rx1) = broadcast::channel::<TunEvent>(1);
    let tun_rx2 = tun_tx.subscribe();

    // NOTE: the code will simply do nothing if not on linux.
    // - The main blocker is that I dont know how my code will be architected for them.
    //   I might need to move the relay connectors to a lib, and make multiple bins.
    #[cfg(target_os = "linux")]
    let _conn = {
        let daemon = DbusDaemon::try_new(tx).await?;

        let conn = connection::Builder::system()?
            .name("me.piguy.lanshare.daemon")?
            .serve_at("/me/piguy/lanshare/daemon", daemon)?
            .build()
            .await?;

        info!("listening on dbus");

        conn
    };

    #[cfg(not(target_os = "linux"))]
    {
        error!("this platform is not supported, the daemon will do nothing");
    }

    // HACK: the ip and netmask needs to be set by the relay after login
    let mut tc = TunController::new();

    let res = tokio::select! {
        res = tc.listen(rx, tun_tx) => res,
        _ = device_task(tun_rx1, tun_rx2) => Ok(()),
    };

    if let Err(error) = &res {
        error!("{error}");
        res?;
    }

    Ok(())
}

#[instrument(skip(rx1, rx2))]
async fn device_task(
    mut rx1: broadcast::Receiver<TunEvent>,
    mut rx2: broadcast::Receiver<TunEvent>,
) {
    loop {
        if let Ok(TunEvent::Up(config)) = rx1.recv().await {
            let mut device = match ::tun::create(&config) {
                Ok(value) => value,
                Err(error) => {
                    error!("{error}");
                    continue;
                }
            };
            let mut buf = [0; 1024];

            loop {
                let amount = match device.read(&mut buf) {
                    Ok(value) => value,
                    Err(error) => {
                        error!("{error}");
                        continue;
                    }
                };
                trace!("read {amount} bytes");

                if let Ok(TunEvent::Down) = rx2.recv().await {
                    debug!("recieved tun down event");
                    break;
                }
            }
        }
    }
}
