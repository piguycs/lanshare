use std::io;

#[macro_use]
extern crate tracing;

pub struct App {
    config: tun::Configuration,
}

impl App {
    pub fn try_new() -> io::Result<Self> {
        let mut config = tun::Configuration::default();

        config
            .tun_name("lanshare0")
            .address("172.16.0.2")
            .netmask("255.255.255.0")
            .mtu(1500) // this seems to be the standard value
            .up();

        config.platform_config(|config| {
            config.ensure_root_privileges(true);
        });

        debug!(?config);
        trace!("created App struct");

        Ok(Self { config })
    }

    pub fn create_dev(&self) -> tun::Result<tun::Device> {
        tun::create(&self.config)
    }
}
