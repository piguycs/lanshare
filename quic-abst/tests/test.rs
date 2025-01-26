use std::net::{Ipv4Addr, SocketAddr};

use bincode::{Decode, Encode};
use quic_abst::handler::Handler;

#[derive(Clone, Copy)]
struct TestHandler;

#[derive(Encode, Decode)]
enum TestInput {
    Good,
    Bad,
}

impl Handler for TestHandler {
    type In = TestInput;
    type Out = Result<(), ()>;

    async fn handle(&self, input: Self::In) -> Self::Out {
        match input {
            TestInput::Good => Ok(()),
            TestInput::Bad => Err(()),
        }
    }
}

#[tokio::test]
async fn client_server_simple() {
    let local_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 5000);

    let handler = TestHandler;
    let (server, cert) = handler.get_server_local(local_addr);

    let client = handler.get_client_local(local_addr, cert);

    tokio::select! {
        _ = server.listen() => panic!("the server must not be the first to close"),
        _ = async {
            let r1 = client.send(TestInput::Good).await;
            let r2 = client.send(TestInput::Bad).await;

            assert!(r1.is_ok());
            assert!(r2.is_err());
        } => (),
    };
}
