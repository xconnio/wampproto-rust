use crate::messages;

use crate::messages::types::Error;
use crate::serializers::helpers::to_message;
use crate::serializers::serializer::Serializer;
use messages::message::Message;
use rmp_serde::{from_slice, to_vec};

#[derive(Debug, Clone)]
pub struct MsgPackSerializer {}

impl Serializer for MsgPackSerializer {
    fn serialize(&self, message: &dyn Message) -> Result<Vec<u8>, Error> {
        to_vec(&message.marshal())
            .map_err(|_| Error::new(format!("failed to serialize message {:?}", message.message_type())))
    }

    fn deserialize(&self, payload: Vec<u8>) -> Result<Box<dyn Message>, Error> {
        let json = from_slice(&payload).map_err(|e| Error::new(format!("failed to deserialize message {e}")))?;
        to_message(json)
    }

    fn is_static(&self) -> bool {
        false
    }
}
