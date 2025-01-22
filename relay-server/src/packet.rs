use std::{collections::HashMap, net::Ipv4Addr, sync::Arc};

use etherparse::Ipv4Header;
use s2n_quic::stream::{ReceiveStream, SendStream};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{Mutex, RwLock},
};

#[instrument(skip(recv))]
pub async fn parsepkt(
    mut recv: ReceiveStream,
    route_table: Arc<RwLock<HashMap<Ipv4Addr, Mutex<SendStream>>>>,
) {
    let mut buf = [0; 4096];
    while let Ok(amount) = recv.read(&mut buf).await {
        let pkt = &buf[..amount];
        match Ipv4Header::from_slice(pkt) {
            Ok((header, _)) => {
                let destination = parse_ipv4(header);
                if let Some(send_stream) = route_table.read().await.get(&destination) {
                    let mut pkt = pkt;
                    let mut send_stream = send_stream.lock().await;
                    if let Err(error) = send_stream.write_buf(&mut pkt).await {
                        error!(?error, "could not send packet to destination: {error}");
                    }
                };
            }
            Err(error) => error!(?error, "could not parse packet: {error}"),
        }
    }
}

pub fn parse_ipv4(header: Ipv4Header) -> Ipv4Addr {
    Ipv4Addr::from_octets(header.destination)
}
