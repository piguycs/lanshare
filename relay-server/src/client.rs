use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};

use bincode::Options;
use s2n_quic::{client::Connect, Client as QuicClient};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{action::Action, error::*, BINCODE, CERT, PUBLIC_SOCKET_ADDR};

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
        let data = BINCODE.serialize(&Action::Login {
            name: username.to_string(),
        })?;

        trace!("trying to send all data");
        send_stream
            .write_all(&data)
            .await
            .map_err(QuicError::from)?;
        trace!("sent login info to server");

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
        let mut buf = vec![];
        let _len = recv_stream.read_buf(&mut buf).await.unwrap();
        let res: (_, _) = BINCODE.deserialize(&buf).unwrap();
        debug!("server assigned ip: {}, mask: {}", res.0, res.1);

        Ok(res)
    }
}
