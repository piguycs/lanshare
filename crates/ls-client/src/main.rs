#[macro_use]
extern crate tracing;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("hello world");

    Ok(())
}
