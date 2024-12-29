#[macro_use]
extern crate tracing;

// use quiche::PROTOCOL_VERSION;

use std::{io, net::UdpSocket};

// https://github.com/google/quiche/blob/71111c/quiche/quic/core/quic_constants.h#L38
// const MAX_DATAGRAM_PACKET: usize = 1350;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let mut buf = [0; u16::MAX as usize];
    //let mut out = 0x546;

    let socket = UdpSocket::bind("127.0.0.1:4433")?;

    //let mut config = quiche::Config::new(PROTOCOL_VERSION)?;

    // we dont need ALPN, we own the client and the server
    // config.set_application_protos(&[])?;

    //let local_addr = socket.local_addr()?;

    loop {
        let (len, from) = match socket.recv_from(&mut buf) {
            Ok(val) => val,
            Err(error) => {
                if error.kind() == io::ErrorKind::WouldBlock {
                    debug!(?error, "loop would block");
                    continue;
                }

                error!("loop failed: {}", error);
                panic!("loop failed: {}", error);
            }
        };

        debug!("got {} bytes from {}", len, from);

        let pkt = &mut buf[..len];
        println!("{}", String::from_utf8(pkt.to_vec())?);
    }
}
