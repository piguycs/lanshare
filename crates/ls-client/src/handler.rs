use s2n_quic::stream::{ReceiveStream, SendStream};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::UnixListener;

pub async fn receive(receive_stream: &mut ReceiveStream) {
    let mut stdout = tokio::io::stdout();
    if let Err(error) = tokio::io::copy(receive_stream, &mut stdout).await {
        error!("Error receiving data: {}", error);
    }
}

pub async fn send(tun_sock: UnixListener, send_stream: &mut SendStream) {
    loop {
        match tun_sock.accept().await {
            Ok((mut tun_stream, _)) => {
                let mut buf = Vec::new();
                match tun_stream.read_to_end(&mut buf).await {
                    Ok(_) => {
                        if let Err(error) = send_stream.write_all(&buf).await {
                            error!("Error sending data to server: {}", error);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Error reading from tun_sock: {}", e);
                        break;
                    }
                }
            }
            Err(e) => {
                error!("Error accepting connection on tun_sock: {}", e);
                break;
            }
        }
    }
}
