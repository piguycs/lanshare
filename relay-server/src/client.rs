use std::{net::SocketAddr, time::Duration};

use s2n_quic::{client::Connect, Client as QuicClient};

pub use crate::action::ServerApi;
use crate::{
    action::{response::*, Action},
    error::*,
    wire, CERT,
};

#[derive(Debug)]
pub struct Client {
    quic_client: QuicClient,
    server_addr: SocketAddr,
    pub timeout: Duration,
}

impl Client {
    #[instrument]
    pub async fn try_new(server_addr: SocketAddr) -> Result<Self> {
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
            server_addr,
            timeout,
        })
    }
}

impl ServerApi for Client {
    #[instrument(skip(self))]
    async fn login(&self, username: &str) -> Result<LoginResp> {
        debug!("trying to log in user");

        let connect = Connect::new(self.server_addr).with_server_name("localhost");

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
        let res: LoginResp = wire::deserialise_stream(&mut recv_stream).await?;
        debug!(
            "server assigned ip: {}, mask: {} for {}",
            res.address, res.netmask, username
        );

        Ok(res)
    }

    #[instrument(skip(self))]
    async fn upgrade_conn(&self) -> Result<()> {
        todo!()
    }
}
