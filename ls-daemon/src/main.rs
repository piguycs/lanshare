#![feature(never_type)]

#[macro_use]
extern crate tracing;

mod daemon;
mod error;
mod tun;

use tokio::sync::mpsc;
use zbus::connection;

use std::io::Read;

use crate::{
    daemon::{DaemonEvent, DbusDaemon},
    tun::TunController,
};

#[tokio::main]
async fn main() -> error::Result<()> {
    tracing_subscriber::fmt::init();
    info!("hello from daemon");

    // channel for recieving events from dbus
    let (tx, rx) = mpsc::channel::<DaemonEvent>(2);
    // channel for recieving tun device from TunController
    let (tun_tx, tun_rx) = mpsc::channel::<Option<::tun::Device>>(2);

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
        _ = device_task(tun_rx) => Ok(()),
    };

    if let Err(error) = &res {
        error!("{error}");
        res?;
    }

    Ok(())
}

// FIXME: unable to put a non blocking loop here
#[instrument(skip(tx))]
async fn device_task(mut tx: mpsc::Receiver<Option<::tun::Device>>) {
    loop {
        match tx.recv().await {
            Some(Some(_device)) => {
                info!("recieved a device, running event loop");

                if tx.try_recv().is_ok() {
                    info!("command to drop device recieved, breaking device event loop");
                    break;
                };
            }
            Some(None) => {
                warn!("no device present to drop");
            }
            None => {
                debug!("channel dropped");
            }
        }
    }
}
