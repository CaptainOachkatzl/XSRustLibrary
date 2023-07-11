use std::fmt::Display;

use displaydoc::Display;
use thiserror::Error;

use crate::{
    connection::Connection,
    cryptography::{
        encryption::{self, Encryption},
        key_exchange::{self, HandshakeMode, KeyExchange},
    },
};

#[derive(Debug, Display, Error)]
pub enum HandshakeError {
    /// Error during key exchange: {0}
    KeyExchange(#[from] key_exchange::Error),
    /// Unable to initialize crypto: {0}
    Crypto(#[from] encryption::Error),
}

#[derive(Debug, Display, Error)]
pub enum TransmissionError {
    /// Underlying connection error: {0}
    Connection(String),
    /// Failed to encrypt message: {0}
    EncryptMessage(encryption::Error),
    /// Failed to decrypt message: {0}
    DecryptMessage(encryption::Error),
}

pub struct EncryptedConnection<Enc, Con> {
    crypto: Enc,
    connection: Con,
}

impl<Enc, Con> EncryptedConnection<Enc, Con>
where
    Enc: Encryption,
    Con: Connection,
    <Con as Connection>::ErrorType: std::fmt::Display,
{
    pub fn with_handshake(mut connection: Con, mut kex: impl KeyExchange, mode: HandshakeMode) -> Result<Self, HandshakeError> {
        let secret = kex.handshake(&mut connection, mode)?;
        let crypto = Enc::initialize(&secret)?;

        Ok(Self {
            connection,
            crypto: *crypto,
        })
    }
}

impl<Enc, Con, E> Connection for EncryptedConnection<Enc, Con>
where
    Enc: Encryption,
    Con: Connection<ErrorType = E>,
    E: Display,
{
    type ErrorType = TransmissionError;

    fn send(&mut self, data: &[u8]) -> Result<(), TransmissionError> {
        let encrypted = self.crypto.encrypt(data).map_err(TransmissionError::DecryptMessage)?;
        self.connection
            .send(&encrypted)
            .map_err(|e| TransmissionError::Connection(e.to_string()))
    }

    fn receive(&mut self) -> Result<Vec<u8>, TransmissionError> {
        let packet = self
            .connection
            .receive()
            .map_err(|e| TransmissionError::Connection(e.to_string()))?;
        self.crypto.decrypt(&packet).map_err(TransmissionError::EncryptMessage)
    }
}
