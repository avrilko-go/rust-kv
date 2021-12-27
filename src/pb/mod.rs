use crate::{CommandRequest, Kvpair, Value};

pub mod abi;

impl CommandRequest {}

impl Kvpair {
    pub fn new(key: impl Into<String>, value: Value) -> Self {
        Self {
            key: key.into(),
            value: Some(value),
        }
    }
}

impl From<(String, Value)> for Kvpair {
    fn from(data: (String, Value)) -> Self {
        Kvpair::new(data.0, data.1)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value {
            value: Some(crate::value::Value::String(s.into())),
        }
    }
}
