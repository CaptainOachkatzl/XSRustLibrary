pub mod curve25519;

use std::fmt::Display;

use displaydoc::Display;
use generic_array::{ArrayLength, GenericArray};
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
    type SecretLength;

    fn handshake<E: Display>(
        &mut self,
        connection: &mut impl Connection<ErrorType = E>,
        mode: HandshakeMode,
    ) -> Result<GenericArray<u8, Self::SecretLength>, Error>
    where
        Self::SecretLength: ArrayLength<u8>;
}
