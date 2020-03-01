use thiserror::Error;

#[derive(Error, Debug)]
pub enum KvsError {
    #[error("standard io error")]
    Io(#[from] std::io::Error),

    #[error("Key not found")]
    NotFound(String),

    #[error("sled error")]
    SledError(#[from] sled::Error),

    #[error("from utf8 error")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, KvsError>;
