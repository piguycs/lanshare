#![feature(never_type)]
#![feature(let_chains)]

#[macro_use]
extern crate tracing;

mod daemon;
mod error;
mod tun;

use tokio::sync::mpsc::{self, error::TryRecvError};
use tun::TunEvent;
use zbus::connection;

use crate::{
    daemon::{DaemonEvent, DbusDaemon},
    tun::TunController,
};

pub const SERVER_ADDR: &str = "192.168.0.26:4433";

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
    let mut stream = None;
    let mut device = None;

    loop {
        match rx.try_recv() {
            Ok(TunEvent::SetRemote(Some(bi))) => stream = Some(bi.split()),
            // gets tun::Configuration, needs to create a device and keep reading in a loop
            Ok(TunEvent::Up(config)) => {
                device = Some(::tun::create_as_async(&config).unwrap().split().unwrap());
            }

            // this means we keep doing what we were doing
            Err(TryRecvError::Empty) if stream.is_some() || device.is_some() => (),
            // no event in queue, we add a little ratelimit here
            Err(TryRecvError::Empty) => tokio::time::sleep(std::time::Duration::from_secs(5)).await,

            // states when the loop must go down
            Ok(TunEvent::Down) => device = None,
            Ok(TunEvent::SetRemote(None)) => stream = None,

            Err(error) => todo!("{error:?}"),
        };

        if let (Some((device_w, device_r)), Some((stream_r, stream_w))) = (&mut device, &mut stream)
        {
            tokio::select! {
                _ = tokio::io::copy(device_r, stream_w) => (),
                _ = tokio::io::copy(stream_r, device_w) => (),
            };
        }
    }
}
