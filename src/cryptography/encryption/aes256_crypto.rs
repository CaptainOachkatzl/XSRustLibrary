use aes_gcm::{
    Aes256Gcm, Key, KeyInit, Nonce,
    aead::{
        AeadInOut,
        consts::{U12, U32},
    },
    aes::cipher::Array,
};

use super::Encryption;

pub const NONCE_SIZE: usize = 12;
pub const TAG_SIZE: usize = 16;

#[derive(Clone)]
pub struct Aes256Crypto {
    crypto: Aes256Gcm,
}

impl Aes256Crypto {
    pub fn new(shared_secret: Key<Aes256Gcm>) -> Self {
        Self {
            crypto: Aes256Gcm::new(&shared_secret),
        }
    }
}

impl Encryption for Aes256Crypto {
    type SecretLength = U32;

    fn encrypt_into(&mut self, data: &[u8], output: &mut Vec<u8>) -> Result<(), super::Error> {
        let mut nonce: Array<u8, U12> = unsafe { Array::uninit().assume_init() };
        rand::fill(&mut nonce);

        // output = [plaintext]
        output.clear();
        output.extend_from_slice(data);

        // output = [ciphertext][tag]
        self.crypto
            .encrypt_in_place(&nonce, b"", output)
            .map_err(|e| super::Error::Encryption(e.to_string()))?;

        // output = [nonce][ciphertext][tag]
        let ciphertext_len = output.len();
        output.resize(ciphertext_len + NONCE_SIZE, 0);
        output.copy_within(0..ciphertext_len, NONCE_SIZE);
        output[..NONCE_SIZE].copy_from_slice(&nonce);
        Ok(())
    }

    fn decrypt_in_place(&mut self, data: &mut Vec<u8>) -> Result<(), super::Error> {
        if data.len() < NONCE_SIZE + TAG_SIZE {
            return Err(super::Error::Encryption(
                "Encrypted message does not contain nonce and tag.".to_string(),
            ));
        }

        let nonce = Nonce::try_from(&data[..NONCE_SIZE])
            .map_err(|err| super::Error::Encryption(format!("Nonce decoding error: {}.", err)))?;

        // data = [ciphertext][tag]
        let ciphertext_len = data.len() - NONCE_SIZE;
        data.copy_within(NONCE_SIZE.., 0);
        data.truncate(ciphertext_len);

        // data = [plaintext]
        self.crypto
            .decrypt_in_place(&nonce, b"", data)
            .map_err(|e| super::Error::Encryption(e.to_string()))?;

        Ok(())
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