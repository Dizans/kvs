use crate::Result;

pub trait KvsEngine{
    /// set key-value pair into database
    fn set(&mut self, key: String, value: String) -> Result<()>;
    
    /// get a value by key.
    /// the result will be None when the key is not exists
    fn get(&mut self, key: String) -> Result<Option<String>>;

    /// remove a value from database
    fn remove(&mut self, key: String) -> Result<()>;
}

mod kvs;
mod sled;

pub use self::kvs::KvStore;
pub use self::sled::SledStore;
