use std::fmt::Display;

use displaydoc::Display;
use generic_array::ArrayLength;
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

/// an encrypted connection that has full modularity regarding its key exchange,
/// the encryption that is used and the underlying connection.
pub struct EncryptedConnection<Enc, Con> {
    crypto: Enc,
    connection: Con,
}

impl<Enc, Con, N> EncryptedConnection<Enc, Con>
where
    Enc: Encryption<SecretLength = N>,
    Con: Connection,
    <Con as Connection>::ErrorType: std::fmt::Display,
{
    /// exchange keys and set up encryption module over the passed in connection.
    /// returns a fully encrypted and immediately usable connection.
    pub fn with_handshake(
        mut connection: Con,
        mut kex: impl KeyExchange<SecretLength = N>,
        mode: HandshakeMode,
    ) -> Result<Self, HandshakeError>
    where
        N: ArrayLength<u8>,
    {
        let secret = kex.handshake(&mut connection, mode)?;
        let crypto = Enc::initialize(&secret)?;

        Ok(Self {
            connection,
            crypto: *crypto,
        })
    }

    /// get the underlying connection to e.g. shut it down.
    /// all traffic that is sent via the connection is NOT ENCRYPTED and readable by attackers.
    pub fn get_underlying_connection(&mut self) -> &mut Con {
        &mut self.connection
    }
}

impl<Enc, Con, E> Connection for EncryptedConnection<Enc, Con>
where
    Enc: Encryption,
    Con: Connection<ErrorType = E>,
    E: Display,
{
    type ErrorType = TransmissionError;

    /// send data that will be encrypted with the crypto module.
    fn send(&mut self, data: &[u8]) -> Result<(), TransmissionError> {
        let encrypted = self.crypto.encrypt(data).map_err(TransmissionError::DecryptMessage)?;
        self.connection
            .send(&encrypted)
            .map_err(|e| TransmissionError::Connection(e.to_string()))
    }

    /// receive data and decrypt it with the crypto module.
    fn receive(&mut self) -> Result<Vec<u8>, TransmissionError> {
        let packet = self
            .connection
            .receive()
            .map_err(|e| TransmissionError::Connection(e.to_string()))?;
        self.crypto.decrypt(&packet).map_err(TransmissionError::EncryptMessage)
    }
}
