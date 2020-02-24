// #![deny(missing_docs)]
#![feature(seek_convenience)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use thiserror::Error;
use std::io::{Seek, SeekFrom};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct KvStore {
    index: HashMap<String, FileOffset>, // key : FileOffset
    path: PathBuf,
    map: HashMap<String, ValuePointer>, // file path : ValuePointer
    writter: BufWriter<File>,
    n_garbage: usize,
}

#[derive(Debug, Clone)]
pub struct FileOffset{
    file_path: String,
    offset: u64,
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
}

impl ValuePointer{
    pub fn new(reader: BufReader<File>) -> Self{ 
        ValuePointer{
            reader,
        }
    }

    pub fn get(&mut self, offset: u64) -> String{
        let s = self.get_line(offset);
        let op: Op = serde_json::from_str(&s).unwrap();
        match op{
            Op::SetRec(_, v) => v,
            _ => unreachable!(),
        }
        
    }

    pub fn get_line(&mut self, offset: u64) -> String{
        let seek = SeekFrom::Start(offset);
        let mut s = String::new();
        self.reader.seek(seek).unwrap();
        self.reader.read_line(&mut s).unwrap();
        s
    }
}

pub type Result<T> = std::result::Result<T, KvsError>;

impl KvStore {
    pub fn new(path:PathBuf, writter: BufWriter<File>) -> Self {
        KvStore {
            index: HashMap::new(),
            path,
            map: HashMap::new(),
            writter,
            n_garbage: 0,
        }
    }
    /// set key-value pair into database
    /// ```rust
    /// use std::error::Error;
    /// use kvs::KvStore;
    ///
    /// fn main()-> Result<(), Box<dyn Error>>{
    ///     let mut kvs = KvStore::open("0.log");
    ///     kv.set("one".to_owned(), "1".to_owned());
    ///     Ok(())
    /// }
    /// ```
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.count_garbage(&key);
        if self.is_too_much_garbage(){
            self.compaction();
        };

        self.set_without_compaction(key, value)
    }

    fn set_without_compaction(&mut self, key: String, value: String)-> Result<()>{
        let file_offset = self.current_offset();
        self.set_with_offset(key.clone(), file_offset);

        let op = Op::SetRec(key.clone(), value.clone());
        let mut log = serde_json::to_string(&op).unwrap();
        log.push('\n');

        self.writter.write_all(&log.into_bytes())?;
        self.writter.flush().unwrap(); // make sure reader can get value immediately after set
        Ok(())
    }

    pub fn set_with_offset(&mut self, key: String, file_offset: FileOffset){
        self.count_garbage(&key);
        self.index.insert(key, file_offset);
    }

    fn count_garbage(&mut self, k: &str){
        if self.index.get(k).is_some(){
            self.n_garbage +=1;
        }
    }

    fn current_offset(&mut self) -> FileOffset{
        let offset = self.writter.stream_position().unwrap();
        let path = self.path.to_str().unwrap().to_owned();
        FileOffset{
            file_path: path, 
            offset
        }
    } 

    /// get a value by key.return None if the key is not exists
    /// ```rust
    /// use std::error::Error;
    /// use kvs::KvStore;
    ///
    /// fn main() -> Result<(), Box<dyn Error>> {
    ///     let mut kv = KvStore::open("0.log");
    ///     kv.set("one".to_owned(), "1".to_owned());
    ///     let v1 = kv.get("one".to_owned());
    ///     assert_eq!(v1, Some(String::from("1")));
    ///     Ok(())
    /// }
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        let value = self.index.get_mut(&key);
        match value{
            Some(file_offset) => {
                let path = file_offset.file_path.clone();
                let offset = file_offset.offset;
                let reader = self.map.get_mut(&path).unwrap();
                let v = reader.get(offset);
                
                Ok(Some(v))
                
            },
            None => Ok(None)
        }
    }

    /// remove a value from database
    pub fn remove(&mut self, key: String) -> Result<()> {
        self.count_garbage(&key);
        self.remove_with_option(key, true)
    }

    pub fn remove_without_log(&mut self, key: String) -> Result<()> {
        self.count_garbage(&key);
        self.remove_with_option(key, false)
    }

    pub fn remove_with_option(&mut self, key: String, with_log: bool) -> Result<()> {
        let exists = self.index.remove(&key);
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
            let path = path.to_str().unwrap().to_owned();
            KvStore::load_file_to_kvs(&mut new_kvs, &path).unwrap();
        }

        Ok(new_kvs)
    }

    fn load_file_to_kvs(kvs: &mut KvStore, path: &str)-> Result<()>{
        let db_file = fs::File::open(&path)?;
        let mut reader = BufReader::new(db_file);
        let mut offset = 0;

        // update index
        loop{
            let mut s = String::new();
            match reader.read_line(&mut s){
                Ok(0) => break,
                Ok(l) => {
                    let file_offset = FileOffset{
                        file_path: path.to_owned(), 
                        offset
                    };
                    offset += l as u64;

                    let op:Op = serde_json::from_str(&s).unwrap();
                    match op{
                        Op::SetRec(k, _) => {kvs.set_with_offset(k, file_offset);},
                        Op::RmRec(k) => {match kvs.remove_without_log(k){
                            Err(_) => continue,
                            Ok(_) => continue,
                        }}
                    }
                },
                Err(_) =>{
                    break;
                }
            }
        }
        
        kvs.map.entry(path.to_owned()).or_insert(ValuePointer::new(reader));
        Ok(())
    }

    fn compaction(&mut self) {
        let old_path = self.path.clone();

        self.path.pop();
        self.path.push(&gen_new_name());

        let file = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .open(&self.path)
                        .unwrap();

        let mut key_offsets: Vec<String> = Vec::new();
        for k in self.index.keys(){
            key_offsets.push(k.clone());
        }

        self.writter = BufWriter::new(file);

        for k in key_offsets.into_iter(){
            let v = self.get(k.clone()).unwrap().unwrap();
            self.index.remove(&k); // make sure set will not increase n_garbage
            self.set_without_compaction(k, v).unwrap();
        }

        let file = fs::File::open(&self.path).unwrap();
        let reader = BufReader::new(file);

        self.map.insert(self.path.to_str().unwrap().to_owned(), ValuePointer::new(reader));
        self.map.remove(old_path.to_str().unwrap());

        fs::remove_file(&old_path).unwrap();
    }
    

    fn is_too_much_garbage(&self) -> bool{
        // self.n_garbage > 1000 * 1000 && self.n_garbage > (self.index.len() / 4)
        self.n_garbage > (self.index.len() / 4)
    }
}

fn gen_new_name() -> String{
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{}.log", timestamp.as_nanos())
}


impl Drop for KvStore{
    fn drop(&mut self){
        if self.is_too_much_garbage(){
            self.compaction();
        }
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
