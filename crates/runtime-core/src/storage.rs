use crate::ResourceID;

pub trait Storage<R: ResourceID>: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Retrieve the value associated with a key.
    /// Returns Ok(None) if the key does not exist.
    fn get(&self, key: &R) -> Result<Option<Vec<u8>>, Self::Error>;

    /// Insert or update the value for a key.
    fn put(&mut self, key: R, value: Vec<u8>) -> Result<(), Self::Error>;

    /// Remove the value associated with a key.
    /// Returns Ok(true) if the key existed and was deleted.
    /// Returns Ok(false) if the key did not exist.
    fn delete(&mut self, key: &R) -> Result<bool, Self::Error>;
}
