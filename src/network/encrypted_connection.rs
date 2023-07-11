use std::{fmt::Display, marker::PhantomData};

use displaydoc::Display;
use thiserror::Error;

use crate::{
    connection::Connection,
    cryptography::{
        encryption::{self, Encryption},
        key_exchange::{HandshakeMode, KeyExchange},
    },
};

#[derive(Debug, Display, Error)]
pub enum Error {
    /// Transmission error: {0}
    Transmission(String),
    /// Crypto initialization error: {0}
    CryptoInitialization(String),
    /// Failed to encrypt message: {0}
    EncryptMessage(encryption::Error),
}

pub struct EncryptedConnection<Enc, Con, E> {
    crypto: Enc,
    connection: Con,
    _marker: PhantomData<E>,
}

impl<Enc, Con, E> EncryptedConnection<Enc, Con, E>
where
    Enc: Encryption,
    Con: Connection<E>,
    E: Display,
{
    pub fn with_handshake(mut connection: Con, kex: impl KeyExchange, mode: HandshakeMode) -> Result<Self, Error> {
        let secret = Self::handshake(&mut connection, kex, mode)?;
        let crypto = Enc::initialize(&secret).map_err(|e| Error::CryptoInitialization(e.to_string()))?;

        Ok(Self {
            connection,
            crypto: *crypto,
            _marker: PhantomData,
        })
    }

    fn handshake(connection: &mut Con, mut kex: impl KeyExchange, mode: HandshakeMode) -> Result<Box<[u8]>, Error> {
        kex.handshake(connection, mode)
            .map_err(|e| Error::CryptoInitialization(e.to_string()))
    }
}

impl<Enc, Con, E> Connection<Error> for EncryptedConnection<Enc, Con, E>
where
    Enc: Encryption,
    Con: Connection<E>,
    E: Display,
{
    fn send(&mut self, data: &[u8]) -> Result<(), Error> {
        let encrypted = self.crypto.encrypt(data).map_err(Error::EncryptMessage)?;
        self.connection.send(&encrypted).map_err(|e| Error::Transmission(e.to_string()))
    }

    fn receive(&mut self) -> Result<Vec<u8>, Error> {
        let packet = self.connection.receive().map_err(|e| Error::Transmission(e.to_string()))?;
        self.crypto.decrypt(&packet).map_err(Error::EncryptMessage)
    }
}
