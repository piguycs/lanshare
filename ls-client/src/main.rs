#[macro_use]
extern crate tracing;

use zbus::{Connection, Result};

use ls_client::*;

// Although we use `tokio` here, you can use any async runtime of choice.
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("hello from ls-client");

    let connection = Connection::system().await?;

    // `proxy` macro creates `MyGreaterProxy` based on `Notifications` trait.
    let proxy = DaemonProxy::new(&connection).await?;

    repl(proxy).await;

    Ok(())
}

async fn repl(proxy: DaemonProxy<'_>) {
    loop {
        let stdin = std::io::stdin();
        let mut buf = String::new();

        if let Err(error) = stdin.read_line(&mut buf) {
            error!("error when reading stdin: {error}");
        }

        let res = match buf.as_str().trim() {
            "up" => proxy.int_up().await,
            "down" => proxy.int_down().await,
            "quit" => break,
            _ => {
                println!("enter command 'up', 'down' or 'quit'");
                continue;
            }
        };

        match res {
            Ok(0) => trace!("repl command executed successfully"),
            Ok(error) => error!("daemon returned err_code_{error}"),
            Err(error) => error!("could not communicate with daemon: {error}"),
        };
    }
}
