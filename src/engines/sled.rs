use crate::{KvsError, Result};
use super::KvsEngine;

use sled::Db;

pub struct SledStore(Db);

impl SledStore{
    pub fn new(db: Db) -> Self{
        SledStore(db)
    }
}

impl KvsEngine for SledStore{
    fn set(&mut self, key: String, value: String) -> Result<()>{
        let tree = &self.0;
        tree.insert(key, value.into_bytes())?;
        tree.flush()?;
        Ok(())
    }
    fn get(&mut self, key: String) -> Result<Option<String>>{
        let tree = &self.0;
        let arr = tree.get(key.into_bytes())?;
        tree.flush()?;
        match arr{
            Some(v) => {
                let s =String::from_utf8(v.to_vec());
                match s{
                    Ok(s) => Ok(Some(s)),
                    Err(e) => Err(KvsError::FromUtf8Error(e)),
                }
            },
            None => Ok(None)
        }
    }

    fn remove(&mut self, key: String) -> Result<()>{
        let tree = &self.0;
        let v = tree.remove(key)?;
        tree.flush()?;
        if v.is_none(){
            return Err(KvsError::NotFound("key not found".to_owned()));
        }
        Ok(())
    }
}
