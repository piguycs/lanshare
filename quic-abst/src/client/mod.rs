#[cfg(test)]
mod test;

use std::{marker::PhantomData, net::SocketAddr};

use bincode::{Decode, Encode};
use quinn::Endpoint;
use tokio::io::AsyncWriteExt;

use crate::BC_CFG;

pub struct Client<In, Out>
where
    In: Decode,
    Out: Encode,
{
    pub endpoint: Endpoint,
    pub addr: SocketAddr,

    pub(crate) marker: PhantomData<(In, Out)>,
}

impl<In, Out> Client<In, Out>
where
    In: Decode + Encode,
    Out: Decode + Encode,
{
    // TODO: server name from cert
    pub async fn send(&self, input: In) -> Out {
        let res = self.endpoint.connect(self.addr, "localhost");

        match res {
            Ok(pending_conn) => {
                if let Ok(conn) = pending_conn.await {
                    // part 1
                    let data = bincode::encode_to_vec(&input, BC_CFG).unwrap();

                    let mut send = conn.open_uni().await.unwrap();
                    send.write_all(&data).await.unwrap();
                    send.shutdown().await.unwrap();
                    drop(send);

                    // part 2
                    let mut buf = [0; 1500];
                    let mut recv = conn.accept_uni().await.unwrap();
                    let amount = recv.read(&mut buf).await.unwrap().unwrap();
                    drop(recv);

                    let data = &buf[..amount];

                    let (out, _len) = bincode::decode_from_slice(data, BC_CFG).unwrap();

                    out
                } else {
                    panic!("what");
                }
            }
            Err(error) => panic!("{error}"),
        }
    }
}
