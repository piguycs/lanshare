//! Daemon for LANShare to manage virtual network devices
//! Creating this allows us to run our client in userspace, as TUN/TAP devices need to be managed
//! by a superuser. This daemon does that job as a systemd service, using unix sockets for IPC

use std::io::Read;

use ls_daemon::App;
use tokio::net::UnixListener;
use tun::Device;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = App::try_new().expect("could not create ipc server");

    let mut dev = app.create_dev().expect("device could not be created");

    let socket = UnixListener::bind("lanshare.sock").unwrap();

    let ipc = coreipc::server::Ipc::from_socket(socket).unwrap();

    let ipc_task = tokio::spawn(async move {
        ipc.run().await;
    });

    tokio::select! {
        e = ipc_task => {e.unwrap()},
        _ = eloop(&mut dev) => {}
    };
}

async fn eloop(dev: &mut Device) {
    let mut buf = [0; 4096];

    loop {
        let pkt_size = dev.read(&mut buf).expect("could not read into buffer");
        let pkt = &buf[..pkt_size];

        tracing::info!("{pkt:?}");
    }
}
