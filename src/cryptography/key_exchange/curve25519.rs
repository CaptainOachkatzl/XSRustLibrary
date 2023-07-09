use rand_core::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey};

use super::KeyExchange;

pub struct Curve25519;

const PUB_KEY_BYTE_SIZE: usize = 32;

impl KeyExchange for Curve25519 {
    fn handshake_active(
        mut send: impl FnMut(&[u8]) -> Result<(), super::Error>,
        mut receive: impl FnMut() -> Result<Box<[u8]>, super::Error>,
    ) -> Result<Box<[u8]>, super::Error> {
        let private_key = EphemeralSecret::new(OsRng);
        let public_key = PublicKey::from(&private_key);

        send(public_key.as_bytes())?;

        let remote_pub_key: [u8; PUB_KEY_BYTE_SIZE] = match receive()?.as_ref().try_into() {
            Ok(v) => v,
            Err(e) => return Err(super::Error::Handshake(e.to_string())),
        };

        // calculate shared secret
        Ok(Box::new(private_key.diffie_hellman(&PublicKey::from(remote_pub_key)).to_bytes()))
    }

    fn handshake_passive(
        mut send: impl FnMut(&[u8]) -> Result<(), super::Error>,
        mut receive: impl FnMut() -> Result<Box<[u8]>, super::Error>,
    ) -> Result<Box<[u8]>, super::Error> {
        let private_key = EphemeralSecret::new(OsRng);
        let public_key = PublicKey::from(&private_key);

        let remote_pub_key: [u8; PUB_KEY_BYTE_SIZE] = match receive()?.as_ref().try_into() {
            Ok(v) => v,
            Err(e) => return Err(super::Error::Handshake(e.to_string())),
        };

        send(public_key.as_bytes())?;

        // calculate shared secret
        Ok(Box::new(private_key.diffie_hellman(&PublicKey::from(remote_pub_key)).to_bytes()))
    }
}
