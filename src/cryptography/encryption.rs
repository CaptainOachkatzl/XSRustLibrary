pub mod aes256_crypto;

use displaydoc::Display;
use thiserror::Error;

#[derive(Debug, Display, Error)]
pub enum Error {
    /// Communication error: {0}
    Communication(String),
    /// Encryption error: {0}
    Encryption(String),
    /// Initialization error: {0}
    Initialization(String),
}

pub trait Encryption {
    fn initialize(shared_secret: &[u8]) -> Result<Box<Self>, Error>;
    fn encrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, Error>;
    fn decrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, Error>;
}
