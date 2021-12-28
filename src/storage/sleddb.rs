use crate::{KvError, Kvpair, Storage, StorageIter, Value};
use sled::{Db, Error, IVec};
use std::path::Path;

#[derive(Debug)]
pub struct Sleddb(Db);

impl Sleddb {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self(sled::open(path).unwrap())
    }

    fn get_full_key(table: &str, key: &str) -> String {
        format!("{}:{}", table, key)
    }

    fn get_table_prefix(table: &str) -> String {
        format!("{}:", table)
    }
}

fn flip<T, E>(x: Option<Result<T, E>>) -> Result<Option<T>, E> {
    x.map_or(Ok(None), |v| v.map(Some))
}

impl Storage for Sleddb {
    fn get(&self, table: &str, key: &str) -> Result<Option<Value>, KvError> {
        let name = Sleddb::get_full_key(table, key);
        let result = self.0.get(name.as_bytes())?.map(|v| v.as_ref().try_into());

        flip(result)
    }

    fn set(&self, table: &str, key: &str, value: Value) -> Result<Option<Value>, KvError> {
        let name = Sleddb::get_full_key(table, key);
        let data: Vec<u8> = value.try_into()?;
        let result = self
            .0
            .insert(name.as_bytes(), data)?
            .map(|v| v.as_ref().try_into());
        flip(result)
    }

    fn contains(&self, table: &str, key: &str) -> Result<bool, KvError> {
        let name = Sleddb::get_full_key(table, key);
        Ok(self.0.contains_key(name.as_bytes())?)
    }

    fn del(&self, table: &str, key: &str) -> Result<Option<Value>, KvError> {
        let name = Sleddb::get_full_key(table, key);
        let result = self
            .0
            .remove(name.as_bytes())?
            .map(|v| v.as_ref().try_into());
        flip(result)
    }

    fn get_all(&self, table: &str) -> Result<Vec<Kvpair>, KvError> {
        let name = Sleddb::get_table_prefix(table);
        Ok(self
            .0
            .scan_prefix(name.as_bytes())
            .map(|v| v.into())
            .collect())
    }

    fn get_iter(&self, table: &str) -> Result<Box<dyn Iterator<Item = Kvpair>>, KvError> {
        let name = Sleddb::get_table_prefix(table);
        let iter = StorageIter::new(self.0.scan_prefix(name.as_bytes()));
        Ok(Box::new(iter))
    }
}

impl From<Result<(IVec, IVec), Error>> for Kvpair {
    fn from(v: Result<(IVec, IVec), Error>) -> Self {
        match v {
            Err(_) => Kvpair::default(),
            Ok((k, v)) => match v.as_ref().try_into() {
                Err(_) => Kvpair::default(),
                Ok(v) => Kvpair::new(ivec_to_key(k.as_ref()), v),
            },
        }
    }
}

fn ivec_to_key(ivec: &[u8]) -> &str {
    let s = std::str::from_utf8(ivec).unwrap();
    let mut iter = s.split(":");
    iter.next();
    iter.next().unwrap()
}
