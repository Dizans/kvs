// #![deny(missing_docs)]
use std::collections::HashMap;
pub struct KvStore {
    map: HashMap<String, String>,
}

impl KvStore {
    pub fn new() -> Self {
        KvStore {
            map: HashMap::new(),
        }
    }
    /// set key-value pair into database
    /// ```rust
    /// use std::error::Error;
    /// use kvs::KvStore;
    ///
    /// fn main()-> Result<(), Box<dyn Error>>{
    ///     let mut kv = KvStore::new();
    ///     kv.set("one".to_owned(), "1".to_owned());
    ///     Ok(())
    /// }
    /// ```
    pub fn set(&mut self, key: String, value: String) {
        self.map.insert(key, value);
    }

    /// get a value by key.return None if the key is not exists
    /// ```rust
    /// use std::error::Error;
    /// use kvs::KvStore;
    ///
    /// fn main() -> Result<(), Box<dyn Error>> {
    ///     let mut kv = KvStore::new();
    ///     kv.set("one".to_owned(), "1".to_owned());
    ///     let v1 = kv.get("one".to_owned());
    ///     assert_eq!(v1, Some(String::from("1")));
    ///     Ok(())
    /// }
    pub fn get(&mut self, key: String) -> Option<String> {
        self.map.get(&key).and_then(|v: &String| Some(v.clone()))
    }

    /// remove a value from database
    pub fn remove(&mut self, key: String) {
        self.map.remove(&key);
    }
}
