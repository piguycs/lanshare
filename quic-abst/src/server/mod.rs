//! # Server
//! This module provides a simple abstraction over a Quic server
//! Initialisation is done using the config best suited for my use case, and a simple handler is
//! proviced to handle connections
//!
//! More complex connections (eg: ones that require direct access to the server) are possible by
//! accessing the inner values of the server struct.
//!
//! # Example
//! ```rust ignore
//! let socket_addr = SocketAddr::from_str("127.0.0.1:5000")?;
//! let (server, cert_der) = Server::try_new_local(socket_addr, MockHandler)?;
//! ```

#[cfg(test)]
mod test;

use std::net::SocketAddr;

use quinn::{rustls, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

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
                let res = self.recv_send_conn(conn).await;

                if let Err(error) = res {
                    eprintln!("{error:?} [ERROR] {error}");
                }
            }
        }
    }

    async fn recv_send_conn(&self, conn: quinn::Connection) -> Result {
        let mut writer = vec![];

        {
            let mut recv = conn.accept_uni().await?;
            self.wrap_handle(&mut recv, &mut writer).await?
        };

        let mut send = conn.open_uni().await?;
        send.write_all(&writer).await?;

        send.shutdown().await?;
        send.stopped().await?;

        Ok(())
    }

    async fn wrap_handle<R, W>(&self, reader: &mut R, writer: &mut W) -> Result
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        let mut buf = [0; 1000];
        if let Ok(amount) = reader.read(&mut buf).await
            && amount > 0
        {
            let (decoded, _len) = bincode::decode_from_slice(&buf[..amount], BC_CFG)?;
            let out = self.handler.handle(decoded);

            let amount = bincode::encode_into_slice(out, &mut buf, BC_CFG)?;
            writer.write_all(&buf[..amount]).await?;

            Ok(())
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
