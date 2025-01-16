use rand::Rng;

use relay_server::{error::*, Server};

#[tokio::main]
async fn main() -> Result {
    tracing_subscriber::fmt::init();

    let mut server = Server::try_new().await?;
    server.accept().await;

    Ok(())
}

fn _ip() -> u32 {
    let mut rng = rand::thread_rng();

    // The first octet is fixed as 25. The rest are random.
    let first_octet = 25u32 << 24; // Shift 25 to the most significant byte.
    let second_octet = (rng.gen_range(0..=255) as u32) << 16;
    let third_octet = (rng.gen_range(0..=255) as u32) << 8;
    let fourth_octet = rng.gen_range(0..=255) as u32;

    // Combine the octets into a single u32.
    first_octet | second_octet | third_octet | fourth_octet
}
