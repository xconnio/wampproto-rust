use crate::messages;
use messages::message::Message;
use messages::types::{Error, Value};
use messages::validator::ValidationSpec;
use std::any::Any;
use std::collections::HashMap;

pub const MESSAGE_TYPE_INTERRUPT: Value = Value::Int(69);
pub const MESSAGE_NAME_INTERRUPT: &str = "INTERRUPT";

pub const VALIDATION_SPEC: ValidationSpec = ValidationSpec {
    min_length: 3,
    max_length: 3,
    name: MESSAGE_NAME_INTERRUPT,
};

#[derive(Debug)]
pub struct Interrupt {
    pub request_id: i64,
    pub options: HashMap<String, Value>,
}

impl Message for Interrupt {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn message_type(&self) -> Value {
        MESSAGE_TYPE_INTERRUPT
    }

    fn marshal(&self) -> Vec<Value> {
        vec![
            MESSAGE_TYPE_INTERRUPT,
            Value::Int(self.request_id),
            Value::Dict(self.options.clone()),
        ]
    }

    fn parse(data: Vec<Value>) -> Result<Box<dyn Message>, Error> {
        VALIDATION_SPEC.validate_message(&data)?;

        if let [_, Value::Int(request_id), Value::Dict(options)] = &data[..] {
            Ok(Box::new(Interrupt {
                request_id: *request_id,
                options: options.clone(),
            }))
        } else {
            Err(VALIDATION_SPEC.err_invalid_message())
        }
    }
}
