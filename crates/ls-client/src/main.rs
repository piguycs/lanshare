//! LANShare Client application

use tokio::{io::AsyncReadExt, net::UnixStream};

#[macro_use]
extern crate tracing;

// static CERT: &str = include_str!("../../../certs/dev.crt");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let mut stream = UnixStream::connect("/run/coreipc/lanshare.sock").await?;
    info!("connected to socket");

    let mut buf = vec![0u8; 1024];

    loop {
        println!("hello world");
        match stream.read(&mut buf).await {
            Ok(0) => {
                info!("Connection closed");
                break;
            }
            Ok(n) => {
                info!("Read {} bytes", n);
            }
            Err(e) => {
                error!("Error reading from stream: {}", e);
                break;
            }
        }
    }

    Ok(())
}
