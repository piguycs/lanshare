use zbus::Result;

#[zbus::proxy(
    interface = "me.piguy.lanshare.daemon1",
    default_service = "me.piguy.lanshare.daemon",
    default_path = "/me/piguy/lanshare/daemon"
)]
pub trait Daemon {
    async fn int_up(&self) -> Result<u64>;
    async fn int_down(&self) -> Result<u64>;
}
