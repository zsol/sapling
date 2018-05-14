use error::Result;
use key::Key;

pub struct Delta {
    pub data: Box<[u8]>,
    pub base: Key,
    pub key: Key,
}

pub trait DataStore {
    fn get(&self, key: &Key) -> Result<Vec<u8>>;
    fn getdeltachain(&self, key: &Key) -> Result<Vec<Delta>>;
}
