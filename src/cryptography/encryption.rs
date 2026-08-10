pub mod aes256_crypto;

use aes_gcm::{aead::array::ArraySize, aes::cipher::Array};
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
    type SecretLength;

    fn initialize(shared_secret: Array<u8, Self::SecretLength>) -> Result<Box<Self>, Error>
    where
        Self::SecretLength: ArraySize;
    fn encrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, Error>;
    fn decrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, Error>;
}
