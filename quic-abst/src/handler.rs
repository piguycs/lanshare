use std::{
    marker::PhantomData,
    net::{Ipv4Addr, SocketAddr},
};

use bincode::{Decode, Encode};
use quinn::{
    rustls::{
        pki_types::{CertificateDer, PrivateKeyDer},
        RootCertStore,
    },
    ClientConfig, Endpoint,
};

use crate::{client::Client, server::Server};

pub trait Handler {
    type In: Decode + Encode;
    type Out: Decode + Encode;

    fn handle(&self, input: Self::In) -> Self::Out;

    fn get_server<C, P>(self, addr: SocketAddr, cert_der: C, priv_key: P) -> Server<Self>
    where
        C: Into<CertificateDer<'static>>,
        P: Into<PrivateKeyDer<'static>>,
        Self: Sized,
    {
        let cert_der = cert_der.into();
        let priv_key = priv_key.into();

        Server::try_new(addr, self, cert_der, priv_key).unwrap()
    }

    fn get_server_local(self, addr: SocketAddr) -> (Server<Self>, CertificateDer<'static>)
    where
        Self: Sized,
    {
        Server::try_new_local(addr, self).unwrap()
    }

    fn get_client_local<C>(&self, addr: SocketAddr, cert_der: C) -> Client<Self::In, Self::Out>
    where
        C: Into<CertificateDer<'static>>,
    {
        let local_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
        let mut endpoint = Endpoint::client(local_addr).unwrap();

        let cert_der = cert_der.into();

        let mut certs = RootCertStore::empty();
        certs.add(cert_der).unwrap();

        let config = ClientConfig::with_root_certificates(certs.into()).unwrap();
        endpoint.set_default_client_config(config);

        Client {
            addr,
            endpoint,
            marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod test {
    use std::net::SocketAddr;

    use super::*;
    use crate::test_utils::*;

    #[rstest::rstest]
    #[tokio::test]
    async fn server_gen(local_addr: SocketAddr, mock_handler: MockHandler, cert: KeyPair) {
        let (cert_der, priv_key) = cert;
        mock_handler.get_server(local_addr, cert_der, priv_key);
    }
}
