use std::{net::Ipv4Addr, sync::Arc};

use tokio::{io::AsyncReadExt, sync::RwLock};

pub mod platform;

const MTU: u16 = 1500;
const TUN_NAME: &str = "lanshare0";
const NETMASK: Ipv4Addr = Ipv4Addr::new(255, 0, 0, 0);

pub type TunController = Arc<RwLock<TunControllerInner>>;

#[derive(Debug)]
pub struct TunControllerInner {
    pub(crate) config: tun::Configuration,
    pub(crate) relay: Option<()>,
}

pub async fn new_controller() -> TunController {
    TunControllerInner::prime().await
}

impl TunControllerInner {
    pub async fn prime() -> Arc<RwLock<Self>> {
        let mut config = tun::Configuration::default();
        config.mtu(MTU).tun_name(TUN_NAME).netmask(NETMASK).up();

        let relay = None;

        let ctl = Arc::new(RwLock::new(Self { config, relay }));

        let ctl_clone = Arc::clone(&ctl);
        tokio::task::spawn(async {
            Self::tester(ctl_clone).await;
        });

        ctl
    }

    async fn tester(ctl: Arc<RwLock<Self>>) {
        loop {
            // get a read lock, extract the config, and then drop the lock
            let config = {
                let ctl = ctl.read().await;
                if ctl.relay.is_some() {
                    let mut config = ctl.config.clone();
                    config.address(Ipv4Addr::new(25, 0, 0, 10));
                    Some(config)
                } else {
                    None
                }
            };

            if let Some(config) = config {
                let mut device = tun::create_as_async(&config).unwrap();
                println!("DEVICE UP");

                let mut buf = [0; MTU as usize];
                loop {
                    match device.read(&mut buf).await {
                        Ok(amount) => {
                            println!("read {amount} bytes");
                            let _pkt = &buf[..amount];
                        }
                        Err(error) => eprintln!("ERROR: {error}"),
                    }

                    if ctl.read().await.relay.is_some() {
                        // device.set_address(new_address);
                        continue;
                    }

                    println!("DEVICE DOWN");
                    break;
                }
            }

            // delay, in order to let other process in our daemon to breathe
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
}
