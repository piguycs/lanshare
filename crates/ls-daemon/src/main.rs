//! Daemon for LANShare to manage virtual network devices
//! Creating this allows us to run our client in userspace, as TUN/TAP devices need to be managed
//! by a superuser. This daemon does that job as a systemd service, using unix sockets for IPC
//!
//! This does not need tokio in it's current state, but the quic client will be moved to this in
//! the future, so as to simplify the client code and allow people to create clients using platform
//! native APIs more easily. (eg: gtk for gnome, qt for kde, imgui for integration with games)

#![feature(str_as_str)]

mod relay;

#[macro_use]
extern crate tracing;

use tokio::net::{UnixListener, UnixStream};

use std::{
    error::Error,
    fs::{self, Permissions},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
};

enum ClientAction {
    Connect,
    Disconnect,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();

    let base_dir = PathBuf::from("/run/lanshare");

    if !base_dir.exists() {
        fs::create_dir_all(&base_dir)?;
    }

    let socket_dir = base_dir.join("lanshare-daemon.sock");
    let socket = UnixListener::bind(&socket_dir)?;
    info!("listening on {socket_dir:?}");

    // allow all users/groups to access the socket directory
    fs::set_permissions(&socket_dir, Permissions::from_mode(0o777))?;
    info!("set permissions for {socket_dir:?} to 777");

    info!("starting event loop");
    loop {
        match socket.accept().await {
            Ok((mut stream, _)) => {
                debug!("spawning a new thread to handle the stream");
                tokio::spawn(async move {
                    if let Err(error) = handle_stream(&mut stream).await {
                        error!("error when handling stream: {error}");
                    }
                });
            }
            Err(error) => {
                error!("could not accept stream: {}", error);
                continue;
            }
        };
    }
}

async fn handle_stream(stream: &mut UnixStream) -> Result<(), Box<dyn Error>> {
    debug!("reading the len for the socket path");

    Ok(())
}
