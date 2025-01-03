//! LANShare Client application

#[macro_use]
extern crate tracing;

// static CERT: &str = include_str!("../../../certs/dev.crt");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("starting client");

    Ok(())
}
