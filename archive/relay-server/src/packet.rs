use std::{collections::HashMap, net::Ipv4Addr, sync::Arc};

use etherparse::err::ipv4::{HeaderError, HeaderSliceError};
use etherparse::Ipv4Header;
use s2n_quic::stream::{ReceiveStream, SendStream};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{Mutex, RwLock},
};

#[instrument(skip(recv, route_table))]
pub async fn parsepkt(
    mut recv: ReceiveStream,
    route_table: Arc<RwLock<HashMap<Ipv4Addr, Mutex<SendStream>>>>,
) {
    debug!(?route_table);
    let mut buf = [0; 4096];
    while let Ok(amount) = recv.read(&mut buf).await {
        let pkt = &buf[..amount];
        match Ipv4Header::from_slice(pkt) {
            Ok((header, _)) => {
                let destination = parse_ipv4(header);
                if let Some(send_stream) = route_table.read().await.get(&destination) {
                    trace!(?route_table, "found stream");
                    let mut pkt = pkt;
                    let mut send_stream = send_stream.lock().await;
                    if let Err(error) = send_stream.write_buf(&mut pkt).await {
                        error!(?error, "could not send packet to destination: {error}");
                    }
                };
            }
            // we ignore ipv6 errors
            Err(HeaderSliceError::Content(HeaderError::UnexpectedVersion {
                version_number: 6,
            })) => (),
            // other errors might be of some concern
            Err(error) => warn!(?error, "could not parse packet: {error}"),
        }
    }
}

pub fn parse_ipv4(header: Ipv4Header) -> Ipv4Addr {
    debug!(?header);
    Ipv4Addr::from_octets(header.destination)
}
