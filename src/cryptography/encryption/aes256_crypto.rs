use aes_gcm::{aead::OsRng, AeadCore, AeadInPlace, Aes256Gcm, KeyInit, Nonce};
use generic_array::{typenum::U32, GenericArray};

use super::Encryption;

pub const NONCE_SIZE: usize = 12;

pub struct Aes256Crypto {
    crypto: Aes256Gcm,
}

impl Aes256Crypto {
    pub fn new(shared_secret: &GenericArray<u8, U32>) -> Self {
        Self {
            crypto: Aes256Gcm::new(shared_secret),
        }
    }
}

impl Encryption for Aes256Crypto {
    type SecretLength = U32;

    fn encrypt(&mut self, mut data: Vec<u8>) -> Result<Vec<u8>, super::Error> {
        let nonce = Aes256Gcm::generate_nonce(OsRng);
        if let Err(e) = self.crypto.encrypt_in_place(&nonce, b"", &mut data) {
            return Err(super::Error::Encryption(e.to_string()));
        }

        // append nonce on the back to avoid moving/copying a lot of memory
        data.extend_from_slice(&nonce);

        Ok(data)
    }

    fn decrypt(&mut self, mut data: Vec<u8>) -> Result<Vec<u8>, super::Error> {
        if data.len() < NONCE_SIZE {
            return Err(super::Error::Encryption("Encrypted message does not contain nonce.".to_string()));
        }

        let nonce_start = data.len() - NONCE_SIZE;
        let nonce_data = data.drain(nonce_start..).collect::<Box<[u8]>>();
        if let Err(e) = self.crypto.decrypt_in_place(Nonce::from_slice(&nonce_data), b"", &mut data) {
            return Err(super::Error::Encryption(e.to_string()));
        }

        Ok(data)
    }

    fn initialize(shared_secret: &GenericArray<u8, U32>) -> Result<Box<Self>, super::Error> {
        Ok(Box::new(Self::new(shared_secret)))
    }
}
