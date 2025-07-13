use crate::messages;
use messages::message::Message;
use messages::types::{Error, Value};
use messages::validator::ValidationSpec;
use std::any::Any;
use std::collections::HashMap;
use std::option::Option;

pub const MESSAGE_TYPE_EVENT: Value = Value::Int(36);
pub const MESSAGE_NAME_EVENT: &str = "EVENT";

const VALIDATION_SPEC: ValidationSpec = ValidationSpec {
    min_length: 4,
    max_length: 6,
    name: MESSAGE_NAME_EVENT,
};

#[derive(Debug)]
pub struct Event {
    pub subscription_id: i64,
    pub publication_id: i64,
    pub details: HashMap<String, Value>,
    pub args: Option<Vec<Value>>,
    pub kwargs: Option<HashMap<String, Value>>,
}

impl Message for Event {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn message_type(&self) -> Value {
        MESSAGE_TYPE_EVENT
    }

    fn marshal(&self) -> Vec<Value> {
        let mut result = vec![
            MESSAGE_TYPE_EVENT,
            Value::Int(self.subscription_id),
            Value::Int(self.publication_id),
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
                Value::Int(subscription_id),
                Value::Int(publication_id),
                Value::Dict(details),
            ] = &data[..]
            {
                Ok(Box::new(Event {
                    subscription_id: *subscription_id,
                    publication_id: *publication_id,
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
                Value::Int(subscription_id),
                Value::Int(publication_id),
                Value::Dict(details),
                Value::List(args),
                Value::Dict(kwargs),
            ] = &data[..]
            {
                Ok(Box::new(Event {
                    subscription_id: *subscription_id,
                    publication_id: *publication_id,
                    details: details.clone(),
                    args: Some(args.clone()),
                    kwargs: Some(kwargs.clone()),
                }))
            } else {
                Err(VALIDATION_SPEC.err_invalid_message())
            }
        } else if let [
            _,
            Value::Int(subscription_id),
            Value::Int(publication_id),
            Value::Dict(details),
            Value::List(args),
        ] = &data[..]
        {
            Ok(Box::new(Event {
                subscription_id: *subscription_id,
                publication_id: *publication_id,
                details: details.clone(),
                args: Some(args.clone()),
                kwargs: None,
            }))
        } else {
            Err(VALIDATION_SPEC.err_invalid_message())
        }
    }
}
