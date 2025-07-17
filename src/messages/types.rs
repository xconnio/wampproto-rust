use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::ops::{Deref, DerefMut};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(transparent)]
pub struct ByteArray(#[serde(with = "serde_bytes")] pub Vec<u8>);

impl Deref for ByteArray {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ByteArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Value {
    Null,
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    List(Vec<Value>),
    Dict(HashMap<String, Value>),
    Bytes(ByteArray),
}

impl Value {
    pub fn str<S: Into<String>>(s: S) -> Self {
        Value::Str(s.into())
    }

    pub fn int<N: Into<i64>>(n: N) -> Self {
        Value::Int(n.into())
    }

    pub fn float<N: Into<f64>>(n: N) -> Self {
        Value::Float(n.into())
    }

    pub fn bool(b: bool) -> Self {
        Value::Bool(b)
    }

    pub fn list<A: Into<Vec<Value>>>(a: A) -> Self {
        Value::List(a.into())
    }

    pub fn dict(o: HashMap<String, Value>) -> Self {
        Value::Dict(o)
    }
}

// Implement From for types
impl From<i64> for Value {
    fn from(val: i64) -> Self {
        Value::Int(val)
    }
}

impl From<f64> for Value {
    fn from(val: f64) -> Self {
        Value::Float(val)
    }
}

impl From<bool> for Value {
    fn from(val: bool) -> Self {
        Value::Bool(val)
    }
}

impl From<&str> for Value {
    fn from(val: &str) -> Self {
        Value::Str(val.to_string())
    }
}

impl From<String> for Value {
    fn from(val: String) -> Self {
        Value::Str(val)
    }
}

impl From<&[u8]> for Value {
    fn from(val: &[u8]) -> Self {
        Value::Bytes(ByteArray(val.to_vec()))
    }
}

impl<const N: usize> From<[u8; N]> for Value {
    fn from(val: [u8; N]) -> Self {
        Value::Bytes(ByteArray(val.to_vec()))
    }
}

impl From<Vec<u8>> for Value {
    fn from(val: Vec<u8>) -> Self {
        Value::Bytes(ByteArray(val.to_vec()))
    }
}

impl From<ByteArray> for Value {
    fn from(val: ByteArray) -> Self {
        Value::Bytes(val)
    }
}

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn new<T: Into<String>>(msg: T) -> Self {
        Error { message: msg.into() }
    }
}

// Implement Display so errors print nicely
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MyError: {}", self.message)
    }
}

// Implement the std::error::Error trait
impl std::error::Error for Error {}
