use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    Bytes(Vec<u8>),
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
