pub mod aes256_crypto;

use displaydoc::Display;
use thiserror::Error;

#[derive(Debug, Display, Error)]
pub enum Error {
    /// Communication error: {0}
    Communication(String),
    /// Encryption error: {0}
    Encryption(String),
}

pub trait Encryption {
    fn encrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, Error>;
    fn decrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, Error>;
}
