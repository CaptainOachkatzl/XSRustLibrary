use std::fmt::Display;

use generic_array::{typenum::U32, GenericArray};
use rand_core::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::connection::Connection;

use super::{Error, KeyExchange};

pub struct Curve25519;

const PUB_KEY_BYTE_SIZE: usize = 32;

impl KeyExchange for Curve25519 {
    type SecretLength = U32;

    fn handshake<E: Display>(
        &mut self,
        connection: &mut impl Connection<ErrorType = E>,
        _mode: super::HandshakeMode,
    ) -> Result<GenericArray<u8, U32>, Error> {
        let private_key = EphemeralSecret::new(OsRng);
        let public_key = PublicKey::from(&private_key);

        connection
            .send(public_key.as_bytes())
            .map_err(|e| Error::Communication(e.to_string()))?;

        let pub_key_data = connection.receive().map_err(|e| Error::Communication(e.to_string()))?;

        let remote_pub_key: [u8; PUB_KEY_BYTE_SIZE] = match pub_key_data.try_into() {
            Ok(v) => v,
            Err(_) => return Err(super::Error::Handshake("Invalid remote public key size".to_string())),
        };

        // calculate shared secret
        Ok(GenericArray::from(
            private_key.diffie_hellman(&PublicKey::from(remote_pub_key)).to_bytes(),
        ))
    }
}
