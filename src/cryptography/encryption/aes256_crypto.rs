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
        let encrypted = match self.crypto.encrypt(&nonce, data) {
            Ok(v) => v,
            Err(e) => return Err(super::Error::Encryption(e.to_string())),
        };

        let mut encrypted_with_nonce = vec![0; NONCE_SIZE + encrypted.len()];
        encrypted_with_nonce[..NONCE_SIZE].copy_from_slice(&nonce);
        encrypted_with_nonce[NONCE_SIZE..].copy_from_slice(&encrypted);
        Ok(encrypted_with_nonce)
    }

    fn decrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, super::Error> {
        if data.len() < NONCE_SIZE {
            return Err(super::Error::Encryption("Encrypted message does not contain nonce.".to_string()));
        }
        let nonce = Nonce::from_slice(&data[..NONCE_SIZE]);
        let decrypted = match self.crypto.decrypt(nonce, &data[NONCE_SIZE..]) {
            Ok(v) => v,
            Err(e) => return Err(super::Error::Encryption(e.to_string())),
        };

        Ok(decrypted)
    }
}
