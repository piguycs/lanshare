//! LANShare Client application

use coreipc::client::Client;

#[macro_use]
extern crate tracing;

// static CERT: &str = include_str!("../../../certs/dev.crt");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("starting client");

    let client = Client::create_client("lanshare").unwrap();
    let _client = client.connect("lanshare.sock").await.unwrap();

    Ok(())
}
