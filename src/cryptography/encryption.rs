pub mod aes256_crypto;

use displaydoc::Display;
use generic_array::{ArrayLength, GenericArray};
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

    fn initialize(shared_secret: &GenericArray<u8, Self::SecretLength>) -> Result<Box<Self>, Error>
    where
        Self::SecretLength: ArrayLength<u8>;
    fn encrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, Error>;
    fn decrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, Error>;
}
