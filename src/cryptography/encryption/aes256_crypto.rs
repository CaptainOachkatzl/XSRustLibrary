use aes_gcm::{
    Aes256Gcm, AesGcm, Key, KeyInit, Nonce,
    aead::{
        Aead,
        consts::{U12, U16, U32},
    },
    aes::cipher::Array,
};

use super::Encryption;

pub const NONCE_SIZE: usize = 12;

#[derive(Clone)]
pub struct Aes256Crypto {
    crypto: Aes256Gcm,
}

impl Aes256Crypto {
    pub fn new(shared_secret: Key<AesGcm<Aes256Gcm, U12, U16>>) -> Self {
        Self {
            crypto: Aes256Gcm::new(&shared_secret),
        }
    }
}

impl Encryption for Aes256Crypto {
    type SecretLength = U32;
    type NonceLength = U12;

    fn encrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, super::Error> {
        let mut nonce: Array<u8, Self::NonceLength> = unsafe { Array::uninit().assume_init() };
        rand::fill(&mut nonce);
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
            return Err(super::Error::Encryption(
                "Encrypted message does not contain nonce.".to_string(),
            ));
        }

        let nonce_start = data.len() - NONCE_SIZE;
        let nonce = Nonce::try_from(&data[nonce_start..])
            .map_err(|err| super::Error::Encryption(format!("Nonce decoding error: {}.", err)))?;
        let decrypted = match self.crypto.decrypt(&nonce, &data[..nonce_start]) {
            Ok(v) => v,
            Err(e) => return Err(super::Error::Encryption(e.to_string())),
        };

        Ok(decrypted)
    }

    fn initialize(
        shared_secret: aes_gcm::aes::cipher::Array<u8, Self::SecretLength>,
    ) -> Result<Box<Self>, super::Error>
    where
        Self::SecretLength: aes_gcm::aead::array::ArraySize,
    {
        Ok(Box::new(Self::new(shared_secret)))
    }
}
