// #![deny(missing_docs)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub struct KvStore {
    map: HashMap<String, String>,
    writter: fs::File,
}

#[derive(Error, Debug)]
pub enum KvsError {
    #[error("standard io error")]
    Io(#[from] std::io::Error),
    #[error("Key not found")]
    NotFound(String),
}

#[derive(Serialize, Deserialize)]
pub enum Op {
    SetRec(String, String),
    RmRec(String),
}

pub type Result<T> = std::result::Result<T, KvsError>;

impl KvStore {
    pub fn new(writter: fs::File) -> Self {
        KvStore {
            map: HashMap::new(),
            writter,
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
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.set_with_option(key, value, true)
    }

    pub fn set_without_log(&mut self, key: String, value: String) -> Result<()> {
        self.set_with_option(key, value, false)
    }

    fn set_with_option(&mut self, key: String, value: String, with_log: bool) -> Result<()> {
        if with_log {
            let op = Op::SetRec(key.clone(), value.clone());
            let log = serde_json::to_string(&op).unwrap();
            self.writter.write_all(&log.into_bytes())?;
        }
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
    pub fn remove(&mut self, key: String) -> Result<()> {
        self.remove_with_option(key, true)
    }

    pub fn remove_without_log(&mut self, key: String) -> Result<()> {
        self.remove_with_option(key, true)
    }

    pub fn remove_with_option(&mut self, key: String, with_log: bool) -> Result<()> {
        let exists = self.map.remove(&key);
        if exists.is_none(){
            return Err(KvsError::NotFound("Key not found".to_owned()));
        }
        if exists.is_some() && with_log {
            let op = Op::RmRec(key);
            let log = serde_json::to_string(&op).unwrap();
            self.writter.write_all(&log.into_bytes())?;
        }
        Ok(())
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        // path is a dir
        //
        // if path does not exists => create this dir
        // if this dir is empty => create a file named 0.log
        // the log files named from 0.log to n.log
        // and each log file has a maximum size
        // get the last log file (n.log), if it's bigger then maximum size
        // then create a new file n+1.log as the writter

        let all_files_path = get_files_path(path).unwrap();
        let mut ops: Vec<Op> = Vec::new();

        let newst_file = all_files_path.last().unwrap().clone();

        for path in all_files_path {
            let db_file = fs::File::open(path)?;
            let mut reader = BufReader::new(db_file);
            let mut log_str = String::new();
            reader.read_to_string(&mut log_str)?;
            let mut logs = serde_json::Deserializer::from_str(&log_str).into_iter::<Op>();
            while let Some(Ok(op)) = logs.next() {
                ops.push(op);
            }
        }

        let append_file = OpenOptions::new().append(true).open(newst_file)?;

        let mut new_kvs = KvStore::new(append_file);
        for op in ops {
            match op {
                Op::SetRec(k, v) => new_kvs.set_without_log(k, v).unwrap(),
                Op::RmRec(k) => new_kvs.remove_without_log(k).unwrap(),
            };
        }

        Ok(new_kvs)
    }
}

/// get all files' PathBuf in the dir which end with ".log"
/// if the dir does not exists
/// then it will create this dir recursively and create
/// a file named '0.log' in this dir and return it
fn get_files_path(path: impl Into<PathBuf>) -> Result<Vec<PathBuf>> {
    let path: PathBuf = path.into();
    if !path.is_dir() {
        fs::create_dir_all(&path)?;
    }

    let mut all_files = Vec::new();

    for entry in fs::read_dir(&path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "log" {
                    all_files.push(path);
                }
            }
        }
    }

    all_files.sort();
    if all_files.is_empty() {
        let mut first_log_file = path.clone();
        first_log_file.push("0.log");
        fs::File::create(first_log_file.as_path()).unwrap();
        all_files.push(first_log_file);
    }
    Ok(all_files)
}
