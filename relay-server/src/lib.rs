use bincode::{Decode, Encode};
use quic_abst::handler::Handler;

pub struct VpnHandler {}

#[derive(Debug, Encode, Decode)]
pub enum HandlerInput {
    Login { username: String, password: String },
}

impl Handler for VpnHandler {
    type In = HandlerInput;
    type Out = Result<(), ()>;

    fn handle(&self, input: Self::In) -> Self::Out {
        match input {
            HandlerInput::Login { .. } => Ok(()),
        }
    }
}
