//! Daemon for LANShare to manage virtual network devices
//! Creating this allows us to run our client in userspace, as TUN/TAP devices need to be managed
//! by a superuser. This daemon does that job as a systemd service, using unix sockets for IPC

#![feature(str_as_str)]

#[macro_use]
extern crate tracing;

use tokio::io::AsyncReadExt;
use tokio::net::{UnixListener, UnixStream};

use std::error::Error;
use std::fs::{self, Permissions};
use std::net::Ipv4Addr;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

const MAX_CLIENTS: usize = 1;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();

    // TODO: ensure root

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
    let mut num_clients = 0;
    loop {
        match socket.accept().await {
            Ok((mut stream, _)) => {
                if num_clients >= MAX_CLIENTS {
                    warn!(?num_clients, ?MAX_CLIENTS, "too many clients connected!");
                    continue;
                }

                num_clients += 1;
                if let Err(error) = handle_stream(&mut stream).await {
                    num_clients -= 1;
                    error!("error when handling stream: {error}");
                }
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
    let len = stream.read_u16().await? as usize;
    let mut buf = vec![0; len];

    let path_len = stream.read_exact(&mut buf).await?;

    let path = String::from_utf8_lossy(&buf[..path_len]);

    UnixStream::connect(path.as_str()).await?;
    info!("connected to client at {path:?}");

    let dev = get_tun_device(stream).await?;

    Ok(())
}

async fn get_tun_device(stream: &mut UnixStream) -> Result<tun::Device, Box<dyn Error>> {
    debug!("reading the bits for ipv4 address");
    let ip_bits = stream.read_u32().await?;
    let ip_addr = Ipv4Addr::from_bits(ip_bits);
    info!("client request provisioning of {ip_addr}");

    debug!("reading the bits for subnet mask");
    let mask_bits = stream.read_u32().await?;
    let subnet = Ipv4Addr::from_bits(mask_bits);
    info!("client requested a subnetmask of {subnet}");

    debug!("reading the bits for destination");
    let dest_bits = stream.read_u32().await?;
    let dest = Ipv4Addr::from_bits(dest_bits);
    info!("client requested a destination of {dest}");

    // we now start creating the tun device
    debug!("starting the creation of the tun device");
    let mut config = tun::Configuration::default();
    config
        .address(ip_addr)
        .netmask(subnet)
        .destination(dest)
        .up();

    Ok(tun::create(&config)?)
}
