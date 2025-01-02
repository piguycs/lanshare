//! Daemon for LANShare to manage virtual network devices
//! Creating this allows us to run our client in userspace, as TUN/TAP devices need to be managed
//! by a superuser. This daemon does that job as a systemd service, using unix sockets for IPC

use std::io::Read;

use ls_daemon::App;
use tokio::join;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = App::try_new().expect("could not create ipc server");

    let _handle = join!(app.ipc.run(), eloop(&app));
}

async fn eloop(app: &App) {
    // TUN-Device from the tun crate
    let mut dev = app.create_dev().expect("device could not be created");

    let mut buf = [0; 4096];

    loop {
        let pkt_size = dev.read(&mut buf).expect("could not read into buffer");
        let pkt = &buf[..pkt_size];

        app.ipc.broadcast(pkt).await;
    }
}
