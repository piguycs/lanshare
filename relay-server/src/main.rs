use std::net::{Ipv4Addr, SocketAddr};

use quic_abst::handler::{CertificateDer, Handler, PemObject, PrivateKeyDer};
use relay_server::VpnHandler;

#[tokio::main]
async fn main() {
    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 4433);

    let cert = CertificateDer::from_pem_file("certs/cert.pem").unwrap();
    let key = PrivateKeyDer::from_pem_file("certs/key.pem").unwrap();

    let server = VpnHandler::new().get_server(addr, cert, key);

    server.listen().await;
}
