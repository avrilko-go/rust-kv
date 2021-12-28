pub mod memory;
pub mod sleddb;

use crate::{KvError, Kvpair, Value};

pub trait Storage: Send + Sync + 'static {
    fn get(&self, table: &str, key: &str) -> Result<Option<Value>, KvError>;

    fn set(&self, table: &str, key: &str, value: Value) -> Result<Option<Value>, KvError>;

    fn contains(&self, table: &str, key: &str) -> Result<bool, KvError>;

    fn del(&self, table: &str, key: &str) -> Result<Option<Value>, KvError>;

    fn get_all(&self, table: &str) -> Result<Vec<Kvpair>, KvError>;

    fn get_iter(&self, table: &str) -> Result<Box<dyn Iterator<Item = Kvpair>>, KvError>;
}

pub struct StorageIter<T> {
    data: T,
}

impl<T> StorageIter<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

impl<T> Iterator for StorageIter<T>
where
    T: Iterator,
    T::Item: Into<Kvpair>,
{
    type Item = Kvpair;

    fn next(&mut self) -> Option<Self::Item> {
        self.data.next().map(|v| v.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemTable;
    use crate::sleddb::Sleddb;
    use tempfile::tempdir;

    #[test]
    fn mem_table_base_interface_should_work() {
        let store = MemTable::new();
        test_base_interface(store);
    }

    #[test]
    fn mem_table_get_all_should_work() {
        let store = MemTable::new();
        test_get_all(store);
    }

    #[test]
    fn mem_table_get_get_iter_should_work() {
        let store = MemTable::new();
        test_get_iter(store);
    }

    #[test]
    fn sleddb_table_base_interface_should_work() {
        let dir = tempdir().unwrap();
        let store = Sleddb::new(dir);
        test_base_interface(store);
    }

    #[test]
    fn sleddb_table_get_all_should_work() {
        let dir = tempdir().unwrap();
        let store = Sleddb::new(dir);
        test_get_all(store);
    }

    #[test]
    fn sleddb_table_get_get_iter_should_work() {
        let dir = tempdir().unwrap();
        let store = Sleddb::new(dir);
        test_get_iter(store);
    }

    fn test_base_interface(store: impl Storage) {
        let v = store.set("t1", "hello".into(), "world".into());
        assert!(v.unwrap().is_none());
        let v1 = store.set("t1", "hello".into(), "world1".into());
        assert_eq!(v1.unwrap(), Some("world".into()));
        let v = store.get("t1", "hello".into());
        assert_eq!(v.unwrap(), Some("world1".into()));
        assert_eq!(None, store.get("t1", "hello1".into()).unwrap());
        assert!(store.get("t2", "hello".into()).unwrap().is_none());

        assert!(store.contains("t1", "hello").unwrap());
        assert!(!store.contains("t1", "hello1").unwrap());
        assert!(!store.contains("t2", "hello").unwrap());

        let v = store.del("t1", "hello");
        assert_eq!(v.unwrap(), Some("world1".into()));

        assert_eq!(None, store.del("t1", "hello1").unwrap());
        assert_eq!(None, store.del("t1", "hello").unwrap());
    }

    fn test_get_all(store: impl Storage) {
        store.set("t2", "k1", "v1".into()).unwrap();
        store.set("t2", "k2", "v2".into()).unwrap();
        let mut data = store.get_all("t2").unwrap();
        data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(
            data,
            vec![
                Kvpair::new("k1", "v1".into()),
                Kvpair::new("k2", "v2".into())
            ]
        )
    }

    fn test_get_iter(store: impl Storage) {
        store.set("t2", "k1", "v1".into()).unwrap();
        store.set("t2", "k2", "v2".into()).unwrap();
        let mut data: Vec<_> = store.get_iter("t2").unwrap().collect();
        data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(
            data,
            vec![
                Kvpair::new("k1", "v1".into()),
                Kvpair::new("k2", "v2".into())
            ]
        )
    }
}
