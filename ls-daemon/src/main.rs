#![feature(never_type)]

#[macro_use]
extern crate tracing;

mod daemon;
mod error;
mod tun;

use std::io::Read;

use ::tun::Configuration as TunConfig;
use tokio::sync::{broadcast, mpsc};
use zbus::connection;

use crate::{
    daemon::{DaemonEvent, DbusDaemon},
    tun::TunController,
};

#[tokio::main]
async fn main() -> error::Result<()> {
    tracing_subscriber::fmt::init();
    info!("hello from daemon");

    // channel for recieving events from dbus
    let (tx, rx) = mpsc::channel::<DaemonEvent>(1);
    // channel for recieving tun device from TunController
    let (tun_tx, tun_rx) = broadcast::channel::<Option<TunConfig>>(1);
    let tun_rx2 = tun_tx.subscribe();

    let greeter = DbusDaemon::new(tx);

    let _conn = connection::Builder::system()?
        .name("me.piguy.lanshare.daemon")?
        .serve_at("/me/piguy/lanshare/daemon", greeter)?
        .build()
        .await?;

    info!("listening on dbus");

    let mut tc = TunController::new();

    let res = tokio::select! {
        res = tc.listen(rx, tun_tx) => res,
        _ = device_task(tun_rx, tun_rx2) => Ok(()),
    };

    if let Err(error) = &res {
        error!("{error}");
        res?;
    }

    Ok(())
}

#[instrument(skip(rx1, rx2))]
async fn device_task(
    mut rx1: broadcast::Receiver<Option<TunConfig>>,
    mut rx2: broadcast::Receiver<Option<TunConfig>>,
) {
    loop {
        if let Ok(Some(config)) = rx1.recv().await {
            let mut device = ::tun::create(&config).unwrap();
            let mut buf = [0; 1024];

            loop {
                let amount = device.read(&mut buf).unwrap();
                trace!("read {amount} bytes");

                if let Ok(None) = rx2.recv().await {
                    debug!("signal down recv");
                    break;
                }
            }
        }
    }
}
