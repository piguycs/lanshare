//! Daemon for LANShare to manage virtual network devices
//! Creating this allows us to run our client in userspace, as TUN/TAP devices need to be managed
//! by a superuser. This daemon does that job as a systemd service, using unix sockets for IPC

use coreipc::IpcServer;
use etherparse::Ipv4Slice;

use std::io::Read;

#[macro_use]
extern crate tracing;

struct App {
    config: tun::Configuration,
}

impl App {
    pub fn new() -> Self {
        let mut config = tun::Configuration::default();

        config
            .tun_name("lanshare0")
            .address("172.16.0.2")
            .netmask("255.255.255.0")
            .up();

        config.platform_config(|config| {
            config.ensure_root_privileges(true);
        });

        debug!(?config);
        trace!("created App struct");

        Self { config }
    }

    pub fn create_dev(&self) -> tun::Result<tun::Device> {
        tun::create(&self.config)
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let ipc_server = IpcServer::create_server("lanshare.sock").unwrap();

    ipc_server.broadcast().await;
}

#[allow(unused)]
fn a() {
    let app = App::new();

    // TUN-Device from the tun crate
    let mut dev = app.create_dev().expect("device could not be created");

    let mut buf = [0; 4096];

    loop {
        let pkt_size = dev.read(&mut buf).expect("could not read into buffer");
        let pkt = &buf[..pkt_size];

        let header = match Ipv4Slice::from_slice(pkt) {
            Ok(value) => value,
            Err(error) => {
                debug!("could not parse packet header: {error}");
                continue;
            }
        };

        info!("{:?}", header);
    }
}
