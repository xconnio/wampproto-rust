use crate::messages;
use messages::message::Message;
use messages::types::{Error, Value};
use messages::validator::ValidationSpec;
use std::any::Any;
use std::collections::HashMap;
use std::option::Option;

pub const MESSAGE_TYPE_INVOCATION: Value = Value::Int(68);
pub const MESSAGE_NAME_INVOCATION: &str = "INVOCATION";

const VALIDATION_SPEC: ValidationSpec = ValidationSpec {
    min_length: 4,
    max_length: 6,
    name: MESSAGE_NAME_INVOCATION,
};

#[derive(Debug)]
pub struct Invocation {
    pub request_id: i64,
    pub registration_id: i64,
    pub details: HashMap<String, Value>,
    pub args: Option<Vec<Value>>,
    pub kwargs: Option<HashMap<String, Value>>,
}

impl Message for Invocation {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn message_type(&self) -> Value {
        MESSAGE_TYPE_INVOCATION
    }

    fn marshal(&self) -> Vec<Value> {
        let mut result = vec![
            MESSAGE_TYPE_INVOCATION,
            Value::Int(self.request_id),
            Value::Int(self.registration_id),
            Value::Dict(self.details.clone()),
        ];

        if let Some(args) = self.args.clone() {
            result.push(Value::List(args));
        }

        if let Some(kwargs) = self.kwargs.clone() {
            if self.args.is_none() {
                result.push(Value::Null)
            }

            result.push(Value::Dict(kwargs));
        }

        result
    }

    fn parse(data: Vec<Value>) -> Result<Box<dyn Message>, Error> {
        VALIDATION_SPEC.validate_message(&data)?;

        if data.len() == VALIDATION_SPEC.min_length {
            if let [
                _,
                Value::Int(request_id),
                Value::Int(registration_id),
                Value::Dict(details),
            ] = &data[..]
            {
                Ok(Box::new(Invocation {
                    request_id: *request_id,
                    registration_id: *registration_id,
                    details: details.clone(),
                    args: None,
                    kwargs: None,
                }))
            } else {
                Err(VALIDATION_SPEC.err_invalid_message())
            }
        } else if data.len() == VALIDATION_SPEC.max_length {
            if let [
                _,
                Value::Int(request_id),
                Value::Int(registration_id),
                Value::Dict(details),
                Value::List(args),
                Value::Dict(kwargs),
            ] = &data[..]
            {
                Ok(Box::new(Invocation {
                    request_id: *request_id,
                    registration_id: *registration_id,
                    details: details.clone(),
                    args: Some(args.clone()),
                    kwargs: Some(kwargs.clone()),
                }))
            } else {
                Err(VALIDATION_SPEC.err_invalid_message())
            }
        } else if let [
            _,
            Value::Int(request_id),
            Value::Int(registration_id),
            Value::Dict(details),
            Value::List(args),
        ] = &data[..]
        {
            Ok(Box::new(Invocation {
                request_id: *request_id,
                registration_id: *registration_id,
                details: details.clone(),
                args: Some(args.clone()),
                kwargs: None,
            }))
        } else {
            Err(VALIDATION_SPEC.err_invalid_message())
        }
    }
}
