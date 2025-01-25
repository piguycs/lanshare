#![feature(never_type)]
#![feature(let_chains)]

#[macro_use]
extern crate tracing;

mod daemon;
mod error;
mod tun;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc::{self, error::TryRecvError},
};
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
async fn device_task_old(mut rx: mpsc::Receiver<TunEvent>) {
    loop {
        match rx.try_recv() {
            // gets a bi-directional stream. basically, calling .split() on it would return a
            // (reader, writer), and you have to read things from the reader and replay them
            // locally on the tun while any outgoing messages from the tun will be written.
            // simply speaking, reads from tun are written to tun, reads from stream are written to tun
            Ok(TunEvent::SetRemote(Some(_))) => todo!(),
            // gets tun::Configuration, needs to create a device and keep reading in a loop
            Ok(TunEvent::Up(_)) => todo!(),
            // device needs to be dropped
            Ok(TunEvent::Down) => todo!(),
            // no event in queue, we add a little ratelimit here
            Err(TryRecvError::Empty) => tokio::time::sleep(std::time::Duration::from_secs(5)).await,
            // todos
            Ok(TunEvent::SetRemote(None)) => todo!("unsetting remote is not supported"),
            Err(error) => todo!("{error:?}"),
        }
    }
}
#[instrument(skip(rx))]
async fn device_task(mut rx: mpsc::Receiver<TunEvent>) {
    let mut tun_device = None; // Placeholder for the TUN device
    let mut remote_stream = None; // Placeholder for the bi-directional stream

    loop {
        match rx.try_recv() {
            Ok(TunEvent::SetRemote(Some(stream))) => {
                // Set the remote stream
                remote_stream = Some(stream);
                info!("Remote stream set");
            }
            Ok(TunEvent::Up(config)) => {
                // Create and configure the TUN device
                tun_device = Some(::tun::create_as_async(&config).unwrap());
                info!("TUN device is up");
            }
            Ok(TunEvent::Down) => {
                // Clean up and close the TUN device
                if let Some(mut device) = tun_device.take() {
                    info!("TUN device starting to shutdown");
                    device.shutdown().await.unwrap();
                    tun_device = None;
                    info!("TUN device is down");
                }
            }
            Ok(TunEvent::SetRemote(None)) => {
                // Handle unsetting the remote stream
                remote_stream = None;
                info!("Remote stream unset");
            }
            Err(TryRecvError::Empty) => {
                // No event in queue, rate limit
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
            Err(error) => {
                error!("Error receiving event: {:?}", error);
            }
        }

        // If we have a remote stream and a TUN device, handle data transfer
        if let (Some(stream), Some(device)) = (remote_stream.as_mut(), tun_device.as_mut()) {
            let mut read_buf = vec![0; 1500];
            let mut write_buf = vec![0; 1500];

            tokio::select! {
                // Handle stream -> device
                stream_result = stream.read(&mut read_buf) => {
                    match stream_result {
                        Ok(0) => {
                            info!("Remote stream closed");
                            remote_stream = None;
                        }
                        Ok(n) => {
                            info!("WRITEEEE");
                            device.write_all(&read_buf[..n]).await.unwrap();
                        }
                        Err(e) => {
                            error!("Error reading from remote stream: {:?}", e);
                        }
                    }
                }
                // Handle device -> stream
                device_result = device.read(&mut write_buf) => {
                    if let Ok(n) = device_result && n > 0 {
                        info!("READDDDD");
                        stream.write_all(&write_buf[..n]).await.unwrap();
                    }
                }
            }
        }
    }
}
