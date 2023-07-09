pub mod curve25519;

use displaydoc::Display;
use thiserror::Error;

#[derive(Debug, Display, Error)]
pub enum Error {
    /// Communication error: {0}
    Communication(String),
    /// Handshake error: {0}
    Handshake(String),
}

pub trait KeyExchange {
    fn handshake_active(
        send: impl FnMut(&[u8]) -> Result<(), Error>,
        receive: impl FnMut() -> Result<Box<[u8]>, Error>,
    ) -> Result<Box<[u8]>, Error>;
    fn handshake_passive(
        send: impl FnMut(&[u8]) -> Result<(), Error>,
        receive: impl FnMut() -> Result<Box<[u8]>, Error>,
    ) -> Result<Box<[u8]>, Error>;
}