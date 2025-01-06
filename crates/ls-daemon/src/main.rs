//! Daemon for LANShare to manage virtual network devices
//! Creating this allows us to run our client in userspace, as TUN/TAP devices need to be managed
//! by a superuser. This daemon does that job as a systemd service, using unix sockets for IPC

use std::fs;

use tokio::net::UnixListener;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    if let Ok(true) = fs::exists("lanshare.sock") {
        fs::remove_file("lanshare.sock").unwrap();
    }

    let socket = UnixListener::bind("lanshare.sock").unwrap();

    let ipc = coreipc::server::Ipc::from_socket(socket).unwrap();

    ipc.run().await; // loop
}
