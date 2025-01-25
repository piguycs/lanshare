use zbus::interface;

use crate::TunController;

#[derive(Debug, zbus::DBusError)]
#[zbus(prefix = "me.piguy.lanshare.daemon")]
pub enum Error {
    #[zbus(error)]
    Zbus(zbus::Error),
    UserLoggedOut,
}

#[derive(Debug)]
pub struct DbusDaemon {
    token: Option<String>,
    tun_controller: TunController,
}

impl DbusDaemon {
    pub fn new(tun_controller: TunController) -> Self {
        Self {
            token: None,
            tun_controller,
        }
    }

    fn get_user(&self) -> Result<&str, Error> {
        match &self.token {
            Some(token) => Ok(token),
            None => Err(Error::UserLoggedOut),
        }
    }
}

#[interface(name = "me.piguy.lanshare.daemon1")]
impl DbusDaemon {
    fn login(&mut self, token: &str) {
        self.token = Some(token.into());
    }

    async fn activate(&self) -> Result<(), Error> {
        let _user = self.get_user()?;

        let mut ctl = self.tun_controller.write().await;
        ctl.relay = Some(());

        Ok(())
    }

    async fn deactivate(&self) -> Result<(), Error> {
        let _user = self.get_user()?;

        let mut ctl = self.tun_controller.write().await;
        ctl.relay = None;

        Ok(())
    }
}
