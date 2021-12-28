use crate::{CommandRequest, KvError, Kvpair, Value};
pub mod abi;
use prost::Message;

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

impl TryFrom<Value> for Vec<u8> {
    type Error = KvError;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        let mut buf = Vec::with_capacity(v.encoded_len());
        v.encode(&mut buf)?;
        Ok(buf)
    }
}

impl TryFrom<&[u8]> for Value {
    type Error = KvError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let msg = Value::decode(value)?;
        Ok(msg)
    }
}
