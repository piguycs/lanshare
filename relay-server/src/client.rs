use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};

use s2n_quic::{client::Connect, Client as QuicClient};

use crate::{action::Action, error::*, wire, CERT, PUBLIC_SOCKET_ADDR};

#[derive(Debug)]
pub struct Client {
    quic_client: QuicClient,
    pub timeout: Duration,
}

impl Client {
    #[instrument]
    pub async fn try_new() -> Result<Self> {
        let quic_client = QuicClient::builder()
            .with_tls(CERT)
            .expect("infailable error: s2n_quic::Client with_tls")
            .with_io("0.0.0.0:0")
            .map_err(QuicError::from)?
            .start()
            .map_err(QuicError::from)?;
        let timeout = Duration::from_secs(30);

        Ok(Self {
            quic_client,
            timeout,
        })
    }

    #[instrument(skip(self))]
    pub async fn login(&self, username: &str) -> Result<(Ipv4Addr, Ipv4Addr)> {
        debug!("trying to log in user");

        // unwrap here is fine, we know that this is almost infailable
        let addr: SocketAddr = PUBLIC_SOCKET_ADDR.parse().unwrap_or_else(|error| {
            error!("{error}");
            panic!("server's address {} is not valid", PUBLIC_SOCKET_ADDR);
        });
        let connect = Connect::new(addr).with_server_name("localhost");

        trace!("trying to connect to the server");
        let mut connection = self
            .quic_client
            .connect(connect)
            .await
            .map_err(QuicError::from)?;

        trace!("trying to open an uni-directional stream");
        let mut send_stream = connection
            .open_send_stream()
            .await
            .map_err(QuicError::from)?;

        trace!("trying to serialize data");
        wire::serialise_stream(
            &mut send_stream,
            &Action::Login {
                name: username.to_string(),
            },
        )
        .await?;
        drop(send_stream);

        let recv_stream = connection
            .accept_receive_stream()
            .await
            .map_err(QuicError::from)?;

        let mut recv_stream = match recv_stream {
            Some(value) => value,
            None => {
                warn!("connection closed prematurely");
                return Err(Error::PrematureClosure);
            }
        };

        trace!("waiting for a response");
        let res: (_, _) = wire::deserialise_stream(&mut recv_stream).await?;
        debug!("server assigned ip: {}, mask: {}", res.0, res.1);

        Ok(res)
    }
}
