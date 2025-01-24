use bincode::Encode;

use super::*;
use crate::error::*;
use crate::test_utils::*;

#[rstest::rstest]
#[case(MockData::Good, Some(Ok(())))]
#[case(MockData::Bad, Some(Err(())))]
#[case("HELLO", None)]
#[tokio::test]
async fn server_handler_wrapper<T>(
    local_addr: SocketAddr,
    mock_handler: MockHandler,
    cert: KeyPair,
    #[case] input: T,
    #[case] output: Option<Result<(), ()>>,
) where
    T: Encode,
{
    let (cert_der, priv_key) = cert;
    let server = MockServer::try_new(local_addr, mock_handler, cert_der, priv_key).unwrap();

    let input = bincode::encode_to_vec(input, BC_CFG).unwrap();
    let mut reader = tokio_test::io::Builder::new().read(&input).build();

    let mut writer = vec![];
    let res = server.wrap_handle(&mut reader, &mut writer).await;

    match (res, output) {
        (Err(Error::DecodeError(_)), None) => (),
        (Ok(_), Some(output)) => {
            let (res, _): (Result<(), ()>, usize) =
                bincode::decode_from_slice(&writer, BC_CFG).unwrap();
            assert_eq!(res, output);
        }
        other => panic!("{other:?}"),
    }
}
