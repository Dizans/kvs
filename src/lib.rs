#![feature(seek_convenience)]
use serde::{Serialize, Deserialize};
use chrono::Local;
use std::io::{self, Write};

mod engines;
mod error;

pub use engines::KvStore;
pub use engines::KvsEngine;
pub use engines::SledStore;
pub use error::{KvsError, Result};

#[derive(Serialize, Deserialize)]
pub enum Command{
    Set(String, String),
    Get(String),
    Rm(String),
}

#[derive(Serialize, Deserialize)]
pub enum Response{
    Null,
    Value(String),
    Error(ServerError),
}

#[derive(Serialize, Deserialize)]
pub enum ServerError{
    NotFound,
    InvalidCommand,
    OtherError,
}

#[derive(PartialEq,Debug)]
pub enum Engine{
    Kvs,
    Sled,
}

pub fn log_init(){
    let env = env_logger::Env::default()
        .filter_or(env_logger::DEFAULT_FILTER_ENV, "trace");
    
    env_logger::Builder::from_env(env)
    .format(|_, record| {
        let mut stderr = io::stderr();
        writeln!(
            stderr,
            "{} {} [{}] {}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.module_path().unwrap_or("<unnamed>"),
            &record.args()
        )
    })
    .init();
}
