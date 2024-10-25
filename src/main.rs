use std::any::Any;
use std::collections::HashMap;
use std::error::Error;

pub trait Message {
    fn type_(&self) -> i32;
    fn parse(&mut self, data: Vec<dyn Any>) -> Result<dyn Message, dyn Error>;
    fn marshal(&self) -> Vec<dyn Any>;
}

pub trait Serializer {
    fn serialize(&self, message: dyn Message) -> Vec<u8>;
    fn deserialize(&self, payload: Vec<u8>) -> dyn Message;
}

pub trait ClientAuthenticator {
    fn auth_method(&self) -> String;
    fn authid(&self) -> String;
    fn authextra(&self) -> HashMap<String, dyn Any>;
}

fn main() {
    println!("Hello, world!");
}
