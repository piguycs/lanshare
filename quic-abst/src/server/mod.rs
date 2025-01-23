//! # Server
//! This module provides a simple abstraction over a Quic server
//! Initialisation is done using the config best suited for my use case, and a simple handler is
//! proviced to handle connections
//!
//! More complex connections (eg: ones that require direct access to the server) are possible by
//! accessing the inner values of the server struct.
//!
//! # Example
//! ```rust
//! let socket_addr = SocketAddr::from_str("127.0.0.1:5000")?;
//! let (server, cert_der) = Server::try_new_local(socket_addr, MockHandler)?;
//! ```

#[cfg(test)]
mod test;

use std::net::SocketAddr;

use quinn::{rustls, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::{error::*, handler::Handler, BC_CFG};

pub struct Server<H: Handler> {
    pub endpoint: quinn::Endpoint,
    pub handler: H,
}

impl<H: Handler> Server<H> {
    // NOTE: this needs end-to end testing not unit testing
    pub async fn listen(&self) {
        while let Some(incoming) = self.endpoint.accept().await {
            if let Ok(conn) = incoming.await {
                // part 1
                let mut recv = conn.accept_uni().await.unwrap();
                let data = self.wrap_handle(&mut recv).await.unwrap();

                // part 2
                let data = bincode::encode_to_vec(data, BC_CFG).unwrap();
                let mut send = conn.open_uni().await.unwrap();
                send.write_all(&data).await.unwrap();
            }
        }
    }

    async fn wrap_handle<R>(&self, recv: &mut R) -> Result<H::Out>
    where
        R: AsyncRead + Unpin,
    {
        let mut buf = [0; 1000];
        if let Ok(amount) = recv.read(&mut buf).await
            && amount > 0
        {
            let (decoded, _len) = bincode::decode_from_slice(&buf[..amount], BC_CFG)?;
            let out = self.handler.handle(decoded);

            Ok(out)
        } else {
            Err(Error::StreamEnd)
        }
    }

    pub fn try_new<C, P>(addr: SocketAddr, handler: H, cert_der: C, priv_key: P) -> Result<Self>
    where
        C: Into<CertificateDer<'static>>,
        P: Into<PrivateKeyDer<'static>>,
    {
        let cert_der = cert_der.into();
        let priv_key = priv_key.into();

        let config = ServerConfig::with_single_cert(vec![cert_der.clone()], priv_key)?;

        let endpoint = quinn::Endpoint::server(config, addr)?;

        Ok(Self { endpoint, handler })
    }

    /// generates a self signed certificate and returns it along with the server
    #[cfg(feature = "rcgen")]
    pub fn try_new_local(addr: SocketAddr, handler: H) -> Result<(Self, CertificateDer<'static>)> {
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;

        let cert_der = CertificateDer::from(cert.cert);
        let priv_key = PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());

        Ok((
            Self::try_new(addr, handler, cert_der.clone(), priv_key)?,
            cert_der,
        ))
    }
}
