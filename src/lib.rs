// #![deny(missing_docs)]
#![feature(seek_convenience)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use thiserror::Error;
use std::io::{Seek, SeekFrom};

pub struct KvStore {
    path: PathBuf,
    map: HashMap<String, ValuePointer>,
    writter: BufWriter<File>,
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

#[derive(Debug)]
pub struct ValuePointer{
    reader: BufReader<File>,
    offset: u64, 
}

impl ValuePointer{
    pub fn new(reader: BufReader<File>, offset: u64) -> Self{ 
        ValuePointer{
            reader,
            offset,
        }
    }

    pub fn get(&mut self) -> String{
        let seek = SeekFrom::Start(self.offset);
        let mut s = String::new();
        self.reader.seek(seek).unwrap();
        self.reader.read_line(&mut s).unwrap();
        let op: Op = serde_json::from_str(&s).unwrap();
        match op{
            Op::SetRec(_, v) => v,
            _ => unreachable!(),
        }
        
    }
}

pub type Result<T> = std::result::Result<T, KvsError>;

impl KvStore {
    pub fn new(path:PathBuf, writter: BufWriter<File>) -> Self {
        KvStore {
            path,
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

    pub fn set_with_pointer(&mut self, key: String, p: ValuePointer){
        self.map.insert(key, p);
    }

    fn set_with_option(&mut self, key: String, value: String, with_log: bool) -> Result<()> {
        let offset = self.writter.stream_position().unwrap();
        let reader = BufReader::new(File::open(&self.path)?);
        let value_pointer = ValuePointer::new(reader, offset);

        if with_log {
            let op = Op::SetRec(key.clone(), value.clone());
            let mut log = serde_json::to_string(&op).unwrap();
            log.push('\n');
            self.writter.write_all(&log.into_bytes())?;
            self.writter.flush().unwrap(); // make sure reader can get value immediately after set
        }
        self.map.insert(key, value_pointer);
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
        let value = self.map.get_mut(&key);
        match value{
            Some(v) => Ok(Some(v.get())),
            None => Ok(None)
        }
    }

    /// remove a value from database
    pub fn remove(&mut self, key: String) -> Result<()> {
        self.remove_with_option(key, true)
    }

    pub fn remove_without_log(&mut self, key: String) -> Result<()> {
        self.remove_with_option(key, false)
    }

    pub fn remove_with_option(&mut self, key: String, with_log: bool) -> Result<()> {
        let exists = self.map.remove(&key);
        if exists.is_none(){
            return Err(KvsError::NotFound("Key not found".to_owned()));
        }

        if with_log {
            let op = Op::RmRec(key);
            let mut log = serde_json::to_string(&op).unwrap();
            log.push('\n');

            self.writter.write_all(&log.into_bytes())?;
        }
        Ok(())
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let all_files_path = get_files_path(path).unwrap();

        let newst_file_path = all_files_path.last().unwrap().clone();

        let mut append_file = OpenOptions::new().write(true).open(&newst_file_path)?;
        append_file.seek(SeekFrom::End(0)).unwrap();
        
        let writter = BufWriter::new(append_file);

        let mut new_kvs = KvStore::new(newst_file_path, writter);
 
        for path in all_files_path {
            let db_file = fs::File::open(&path)?;
            let mut reader = BufReader::new(db_file);
            let mut offset = 0;
        
            loop{
                let mut s = String::new();
                match reader.read_line(&mut s){
                    Ok(0) => break,
                    Ok(l) => {
                        let r = BufReader::new(File::open(&path)?);
                        let p = ValuePointer::new(r, offset as u64);
                        offset +=l;

                        let op:Op = serde_json::from_str(&s).unwrap();
                        match op{
                            Op::SetRec(k, _) => {new_kvs.set_with_pointer(k, p);},
                            Op::RmRec(k) => {new_kvs.remove_without_log(k);}
                        }
                    },
                    Err(_) =>{
                        break;
                    }
                }
            }
            // let mut log_str = String::new();
            // reader.read_to_string(&mut log_str)?;
            // let mut logs = serde_json::Deserializer::from_str(&log_str).into_iter::<Op>();
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
