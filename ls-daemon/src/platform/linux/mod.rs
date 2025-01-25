mod dbus;

use zbus::connection;

use crate::TunController;
use dbus::DbusDaemon;

pub struct LinuxPlatform {
    _conn: connection::Connection,
}

impl LinuxPlatform {
    pub async fn new(tun_controller: TunController) -> Result<Self, zbus::Error> {
        let daemon = DbusDaemon::new(tun_controller);

        let _conn = connection::Builder::system()?
            .name("me.piguy.lanshare.daemon")?
            .serve_at("/me/piguy/lanshare/daemon", daemon)?
            .build()
            .await?;

        Ok(Self { _conn })
    }
}
