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

    let mut buf = vec![0u8; u16::MAX as usize];

    loop {
        match stream.read(&mut buf).await {
            Ok(0) => {
                info!("Connection closed");
                break;
            }
            Ok(n) => {
                let mut pos = 0;
                while pos + 2 <= n {
                    let len = u16::from_be_bytes([buf[pos], buf[pos + 1]]) as usize;
                    if pos + 2 + len <= n {
                        let data = &buf[pos + 2..pos + 2 + len];
                        info!("Read {} bytes, processing {} bytes of data", n, len);
                        info!("{:?}", etherparse::Ipv4Slice::from_slice(data));
                        pos += 2 + len; // Move to the next packet
                    } else {
                        error!(
                            "Invalid length: {} exceeds available data: {}",
                            len,
                            n - pos - 2
                        );
                        break;
                    }
                }
            }
            Err(e) => {
                error!("Error reading from stream: {}", e);
                break;
            }
        }
    }

    Ok(())
}
