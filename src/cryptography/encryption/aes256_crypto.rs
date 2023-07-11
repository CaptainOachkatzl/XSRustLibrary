use aes_gcm::{
    aead::{AeadMut, OsRng},
    AeadCore, Aes256Gcm, KeyInit, Nonce,
};

use super::Encryption;

pub const SHARED_SECRET_SIZE: usize = 32;
pub const NONCE_SIZE: usize = 12;

pub struct Aes256Crypto {
    crypto: Aes256Gcm,
}

impl Aes256Crypto {
    pub fn new(secret: &[u8; SHARED_SECRET_SIZE]) -> Self {
        Self {
            crypto: Aes256Gcm::new(secret.into()),
        }
    }
}

impl Encryption for Aes256Crypto {
    fn encrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, super::Error> {
        let rng = OsRng;
        let nonce = Aes256Gcm::generate_nonce(rng);
        let mut encrypted = match self.crypto.encrypt(&nonce, data) {
            Ok(v) => v,
            Err(e) => return Err(super::Error::Encryption(e.to_string())),
        };

        // append nonce on the back to avoid moving/copying a lot of memory
        encrypted.extend_from_slice(&nonce);

        Ok(encrypted)
    }

    fn decrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, super::Error> {
        if data.len() < NONCE_SIZE {
            return Err(super::Error::Encryption("Encrypted message does not contain nonce.".to_string()));
        }

        let nonce_start = data.len() - NONCE_SIZE;
        let nonce = Nonce::from_slice(&data[nonce_start..]);
        let decrypted = match self.crypto.decrypt(nonce, &data[..nonce_start]) {
            Ok(v) => v,
            Err(e) => return Err(super::Error::Encryption(e.to_string())),
        };

        Ok(decrypted)
    }

    fn initialize(shared_secret: &[u8]) -> Result<Box<Self>, super::Error> {
        let sized_secret: [u8; SHARED_SECRET_SIZE] = shared_secret
            .try_into()
            .map_err(|_| super::Error::Initialization("Invalid secret size".to_string()))?;
        Ok(Box::new(Self::new(&sized_secret)))
    }
}
