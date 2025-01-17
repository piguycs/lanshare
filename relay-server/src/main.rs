use relay_server::{error::*, Server};

#[tokio::main]
async fn main() -> Result {
    tracing_subscriber::fmt::init();

    let mut server = Server::try_new().await?;
    server.accept().await;

    Ok(())
}
