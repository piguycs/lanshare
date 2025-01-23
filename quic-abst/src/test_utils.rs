use std::net::{Ipv4Addr, SocketAddr};

use bincode::{Decode, Encode};
use quinn::rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

use crate::handler::Handler;
use crate::server::Server;

#[derive(Encode, Decode)]
pub enum MockData {
    Good,
    Bad,
}

pub struct MockHandler;
impl Handler for MockHandler {
    type In = MockData;
    type Out = Result<(), ()>;

    fn handle(&self, input: Self::In) -> Self::Out {
        match input {
            MockData::Good => Ok(()),
            MockData::Bad => Err(()),
        }
    }
}

pub type MockServer = Server<MockHandler>;

pub type KeyPair = (CertificateDer<'static>, PrivateKeyDer<'static>);
#[rstest::fixture]
pub fn cert() -> KeyPair {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();

    let cert_der = CertificateDer::from(cert.cert);
    let priv_key = PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());

    (cert_der, priv_key.into())
}

#[rstest::fixture]
pub fn local_addr() -> SocketAddr {
    // port#0 means the os will assign a random port
    SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0)
}

#[rstest::fixture]
pub fn mock_handler() -> MockHandler {
    MockHandler
}
