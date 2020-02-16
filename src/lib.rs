// #![deny(missing_docs)]
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use std::io::BufReader;
use std::io::prelude::*;
use std::fs::OpenOptions;

pub struct KvStore {
    map: HashMap<String, String>,
}

#[derive(Error, Debug)]
pub enum KvsError{
    #[error("standard io error")]
    Io(#[from] std::io::Error),
    #[error("other error")]
    NotFound(String),
}

#[derive(Serialize, Deserialize)]
pub enum Op{
    SetRec(String, String),
    RmRec(String),
}

pub type Result<T> = std::result::Result<T, KvsError>;

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
    pub fn set(&mut self, key: String, value: String) -> Result<()>{
        self.map.insert(key, value);
        Ok(())
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
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        Ok(self.map.get(&key).and_then(|v: &String| Some(v.clone())))
    }

    /// remove a value from database
    pub fn remove(&mut self, key: String) -> Result<()>{
        self.map.remove(&key);
        Ok(())
    }

    pub fn open(path: &Path) -> Result<Self>{
        let db_file = OpenOptions::new()
                        .read(true)
                        .create(true)
                        .open(path)?;
        let mut new_kvs = KvStore::new();

        let mut reader = BufReader::new(db_file); 
        let mut log_str = String::new();
        reader.read_to_string(&mut log_str)?;
        
        let mut logs = serde_json::Deserializer::from_str(&log_str).into_iter::<Op>();
        while let Some(Ok(op)) = logs.next() {
            match op{
                Op::SetRec(k, v)=> new_kvs.set(k, v)?,
                Op::RmRec(k) => new_kvs.remove(k)?,
            }
        }
        Ok(new_kvs)
    }
}
