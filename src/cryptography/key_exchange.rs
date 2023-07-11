pub mod curve25519;

use displaydoc::Display;
use thiserror::Error;

use crate::connection::Connection;

pub enum HandshakeMode {
    Client,
    Server,
}

#[derive(Debug, Display, Error)]
pub enum Error {
    /// Communication error: {0}
    Communication(String),
    /// Handshake error: {0}
    Handshake(String),
}

pub trait KeyExchange {
    fn handshake<E>(connection: &mut impl Connection<E>, mode: HandshakeMode) -> Result<Box<[u8]>, Error>
    where
        E: std::fmt::Display;
}
