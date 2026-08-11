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

    /// encrypt data and return the result as a new buffer.
    fn encrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let mut output = Vec::new();
        self.encrypt_into(data, &mut output)?;
        Ok(output)
    }

    /// encrypt data and write the result into the provided buffer, reusing its allocation.
    fn encrypt_into(&mut self, data: &[u8], output: &mut Vec<u8>) -> Result<(), Error> {
        output.clear();
        output.extend_from_slice(data);
        self.encrypt_in_place(output)
    }

    /// encrypt data in place, reusing the buffer's allocation.
    fn encrypt_in_place(&mut self, data: &mut Vec<u8>) -> Result<(), Error>;

    /// decrypt data and return the result as a new buffer.
    fn decrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let mut output = Vec::new();
        self.decrypt_into(data, &mut output)?;
        Ok(output)
    }

    /// decrypt data and write the result into the provided buffer, reusing its allocation.
    fn decrypt_into(&mut self, data: &[u8], output: &mut Vec<u8>) -> Result<(), Error> {
        output.clear();
        output.extend_from_slice(data);
        self.decrypt_in_place(output)
    }

    /// decrypt data in place, reusing the buffer's allocation.
    fn decrypt_in_place(&mut self, data: &mut Vec<u8>) -> Result<(), Error>;
}
