use rand_core::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey};

use super::key_exchange::{self, KeyExchange};

pub struct Curve25519;

const PUB_KEY_BYTE_SIZE: usize = 32;

impl KeyExchange for Curve25519 {
    fn handshake_active(
        mut send: impl FnMut(&[u8]) -> Result<(), key_exchange::Error>,
        mut receive: impl FnMut() -> Result<Box<[u8]>, key_exchange::Error>,
    ) -> Result<Box<[u8]>, key_exchange::Error> {
        let private_key = EphemeralSecret::new(OsRng);
        let public_key = PublicKey::from(&private_key);

        send(public_key.as_bytes())?;

        let remote_pub_key: [u8; PUB_KEY_BYTE_SIZE] = match receive()?.as_ref().try_into() {
            Ok(v) => v,
            Err(e) => return Err(key_exchange::Error::Handshake(e.to_string())),
        };

        // calculate shared secret
        Ok(Box::new(private_key.diffie_hellman(&PublicKey::from(remote_pub_key)).to_bytes()))
    }

    fn handshake_passive(
        mut send: impl FnMut(&[u8]) -> Result<(), key_exchange::Error>,
        mut receive: impl FnMut() -> Result<Box<[u8]>, key_exchange::Error>,
    ) -> Result<Box<[u8]>, key_exchange::Error> {
        let private_key = EphemeralSecret::new(OsRng);
        let public_key = PublicKey::from(&private_key);

        let remote_pub_key: [u8; PUB_KEY_BYTE_SIZE] = match receive()?.as_ref().try_into() {
            Ok(v) => v,
            Err(e) => return Err(key_exchange::Error::Handshake(e.to_string())),
        };

        send(public_key.as_bytes())?;

        // calculate shared secret
        Ok(Box::new(private_key.diffie_hellman(&PublicKey::from(remote_pub_key)).to_bytes()))
    }
}
