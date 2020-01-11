pub struct KvStore;

impl KvStore{
    pub fn new() -> Self{
        KvStore{}
    }

    pub fn set(&mut self, key: String, value: String){
        unimplemented!()
    }

    pub fn get(&mut self, key: String) -> Option<String>{
        unimplemented!()
    }

    pub fn remove(&mut self, key: String){
        unimplemented!()
    }
}
