//! LANShare Client application

#![feature(ip_from)]

mod handler;

use rand::Rng;
use tokio::{
    io::AsyncWriteExt,
    net::{UnixListener, UnixStream},
};

use std::{
    env,
    error::Error,
    io,
    net::{Ipv4Addr, SocketAddr},
    os::unix::ffi::OsStrExt,
    path::PathBuf,
    sync::LazyLock,
};

#[macro_use]
extern crate tracing;

const CERT: &str = include_str!("../../../certs/cert.pem");
const SERVER_ADDR: &str = "127.0.0.1:4433";

static RUNTIME_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from(env::var("XDG_RUNTIME_DIR").unwrap_or("/tmp".to_string())));

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // we setup a basic s2n-quic client, this is for communicating with the relay-server
    let client = s2n_quic::Client::builder()
        .with_tls(CERT)?
        .with_io("0.0.0.0:0")?
        .start()?;

    // tun_sock is the Unix socket where the client will listen for data from the daemon
    // we generate a random ID in the range of 0-16777215 (0xFFFFFF)
    let rand_id: u32 = rand::thread_rng().gen_range(0..=0xFFFFFF);
    let socket_path = RUNTIME_DIR.join(format!("lanshare-client-{rand_id:06X}.sock"));
    debug!("client listening on {socket_path:?}");
    let tun_sock = UnixListener::bind(&socket_path)?;

    let mut server = get_server().await?;

    // we send a "hello" to the server with our socket address
    send_hello(&mut server, socket_path).await?;

    // TODO: modify SERVER_ADDR during runtime
    let addr: SocketAddr = SERVER_ADDR.parse()?;
    let connect = s2n_quic::client::Connect::new(addr).with_server_name("localhost");

    let mut connection = client.connect(connect).await?;
    connection.keep_alive(true)?;
    info!("connected to relay server at {}", addr);

    let stream = connection.open_bidirectional_stream().await?;
    let (mut receive_stream, mut send_stream) = stream.split();

    let receive_handle = tokio::spawn(async move { handler::receive(&mut receive_stream).await });
    let send_handle = tokio::spawn(async move { handler::send(tun_sock, &mut send_stream).await });

    tokio::try_join!(receive_handle, send_handle).unwrap();

    Ok(())
}

async fn get_server() -> Result<UnixStream, Box<dyn Error>> {
    match UnixStream::connect("/run/lanshare/lanshare-daemon.sock").await {
        Ok(v) => Ok(v),
        Err(error) => {
            match error.kind() {
                std::io::ErrorKind::NotFound => {
                    error!("Could not connect to the daemon. Is the daemon running?")
                }
                std::io::ErrorKind::PermissionDenied => {
                    error!("Permission denied when trying to access daemon.")
                }
                kind => {
                    error!("Error when accessing the daemon. {kind}")
                }
            };

            Err(error.into())
        }
    }
}

async fn send_hello(server: &mut UnixStream, socket_path: PathBuf) -> Result<(), io::Error> {
    let binding = socket_path.into_os_string();
    let msg = binding.as_bytes();
    let msg_len = msg.len() as u16;

    let len_as_bytes = &msg_len.to_be_bytes();
    // logically speaking, this should never trigger
    assert_eq!(len_as_bytes.len(), 2, "could not serialize properly");

    let mut buffer = Vec::with_capacity(2 + msg.len());
    buffer.extend_from_slice(len_as_bytes);
    buffer.extend_from_slice(msg);

    server.write_all(&buffer).await?;

    // tun address
    // TODO: get this address from the control server
    server
        .write_u32(u32::from(Ipv4Addr::new(25, 0, 0, 2)))
        .await?;

    // subnet mask
    server
        .write_u32(u32::from(Ipv4Addr::new(255, 0, 0, 0)))
        .await?;

    Ok(())
}
