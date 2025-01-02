use std::io;

use coreipc::IpcServer;

#[macro_use]
extern crate tracing;

pub struct App {
    config: tun::Configuration,
    pub ipc: IpcServer,
}

impl App {
    pub fn try_new() -> io::Result<Self> {
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

        let ipc = IpcServer::create_server("lanshare.sock")?;

        Ok(Self { config, ipc })
    }

    pub fn create_dev(&self) -> tun::Result<tun::Device> {
        tun::create(&self.config)
    }
}
