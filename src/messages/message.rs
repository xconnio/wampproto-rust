use crate::messages;

use messages::types::{Error, Value};
use std::any::Any;

pub trait Message {
    fn as_any(&self) -> &dyn Any;
    fn message_type(&self) -> Value;
    fn marshal(&self) -> Vec<Value>;
    fn parse(data: Vec<Value>) -> Result<Box<dyn Message>, Error>
    where
        Self: Sized;
}
