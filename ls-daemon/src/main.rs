#![allow(unused)]

use ls_daemon::new_controller;
#[cfg(target_os = "linux")]
use ls_daemon::platform::linux::LinuxPlatform as Platform;

#[tokio::main]
async fn main() {
    let tun_controller = new_controller().await;

    let platform = Platform::new(tun_controller).await;

    std::future::pending::<()>().await;
}
