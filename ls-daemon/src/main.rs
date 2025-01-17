#![feature(never_type)]

#[macro_use]
extern crate tracing;

mod daemon;
mod error;
mod tun;
use tokio::{
    io::AsyncReadExt,
    sync::mpsc::{self, error::TryRecvError},
};
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
    let (tun_tx, tun_rx1) = mpsc::channel::<TunEvent>(1);

    // NOTE: the code will simply do nothing if not on linux.
    // - The main blocker is that I dont know how my code will be architected for them.
    //   I might need to move the relay connectors to a lib, and make multiple bins.
    // - Dbus is supported outside linux, but I would prefer to use platform-specific APIs
    //   - COM (or whatever) for Windows
    //   - XPC for SoyOS
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
        error!("this platform is not supported (yet), the daemon will do nothing");
    }

    let mut tc = TunController::new();

    let res = tokio::select! {
        res = tc.listen(rx, tun_tx) => res,
        _ = device_task(tun_rx1) => Ok(()),
    };

    if let Err(error) = &res {
        error!(?error, "{error}");
        res?;
    }

    Ok(())
}

#[instrument(skip(rx))]
async fn device_task(mut rx: mpsc::Receiver<TunEvent>) {
    loop {
        let config = match rx.recv().await {
            Some(TunEvent::Up(config)) => config,
            Some(TunEvent::Down) => {
                warn!("TUN interface is already down");
                continue;
            }
            None => return error!("channel closed"),
        };

        let mut device = ::tun::create_as_async(&config).unwrap();

        let mut buf = [0; 4096];
        loop {
            let amount = device.read(&mut buf).await.unwrap();
            info!("read {amount} bytes");

            match rx.try_recv() {
                Ok(TunEvent::Down) => break,
                Ok(TunEvent::Up(_)) => warn!("TUN interface is already up"),
                Err(TryRecvError::Empty) => (), // happy case
                Err(error) => error!(?error, "(probably) nonfatal error: {error}"),
            }
        }
    }
}
