use std::{fmt::write, net::SocketAddr, time::Duration};

use s2n_quic::{client::Connect, Client as QuicClient, Connection};
use serde::de::DeserializeOwned;

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

    #[instrument(skip(self))]
    async fn get_connection(&self) -> Result<Connection> {
        let connect = Connect::new(self.server_addr).with_server_name("localhost");

        trace!("trying to connect to the server");
        let connection = self
            .quic_client
            .connect(connect)
            .await
            .map_err(QuicError::from)?;

        Ok(connection)
    }

    #[instrument(skip(self, connection))]
    async fn send_action(&self, connection: &mut Connection, action: Action) -> Result<()> {
        trace!("trying to open an uni-directional stream");
        let mut send_stream = connection
            .open_send_stream()
            .await
            .map_err(QuicError::from)?;

        trace!("trying to serialize data");
        wire::serialise_stream(&mut send_stream, &action).await?;

        Ok(())
    }

    #[instrument(skip(self, connection))]
    async fn receive_data<T: DeserializeOwned>(&self, connection: &mut Connection) -> Result<T> {
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
        let res = wire::deserialise_stream(&mut recv_stream).await?;

        Ok(res)
    }

    async fn send_and_recv<T: DeserializeOwned>(
        &self,
        connection: &mut Connection,
        action: Action,
    ) -> Result<T> {
        self.send_action(connection, action).await?;
        let res = self.receive_data(connection).await?;
        Ok(res)
    }
}

impl ServerApi for Client {
    #[instrument(skip(self))]
    async fn login(&self, username: &str) -> Result<LoginResp> {
        trace!("trying to log in user");

        let mut connection = self.get_connection().await?;

        let action = Action::Login {
            name: username.to_string(),
        };

        let res: LoginResp = self.send_and_recv(&mut connection, action).await?;

        debug!(
            "server assigned ip: {}, mask: {} for {}",
            res.address, res.netmask, username
        );

        Ok(res)
    }

    #[instrument(skip(self))]
    async fn upgrade_conn(&self, token: &str) -> Result<()> {
        let mut connection = self.get_connection().await?;

        info!("negotiating parameters for a bi-directional stream over an uni-directional channel");
        self.send_action(&mut connection, Action::UpgradeConn)
            .await?;
        info!("one-way connection has been dropped upgrading to a bi-directional one");

        let res = connection.accept_bidirectional_stream().await;

        let bi = match res {
            Ok(Some(value)) => value,
            Ok(None) => todo!(),
            Err(_error) => todo!(),
        };
        debug!(?bi);

        if let Err(error) = connection.keep_alive(true) {
            error!("Connection::keep_alive failed: {error}");
        }

        let (mut recv, mut send) = bi.split();

        let handle = tokio::spawn(async {
            for i in 0..10 {
                println!("{i}");
            }
        });
        println!("hello");
        handle.await.unwrap();

        Ok(())
    }
}
