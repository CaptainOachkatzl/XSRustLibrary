use std::fmt::Display;

use aes_gcm::aead::array::ArraySize;
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

/// an encrypted connection that has full modularity regarding its key exchange,
/// the encryption that is used and the underlying connection.
pub struct EncryptedConnection<Enc, Con> {
    crypto: Enc,
    connection: Con,
    send_buffer: Vec<u8>,
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
        N: ArraySize,
    {
        let secret = kex.handshake(&mut connection, mode)?;
        let crypto = Enc::initialize(secret)?;

        Ok(Self {
            connection,
            crypto: *crypto,
            send_buffer: Vec::new(),
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
    Enc: Encryption + Clone,
    Con: Connection<ErrorType = E>,
    E: Display,
{
    type ErrorType = TransmissionError;

    /// send data that will be encrypted with the crypto module.
    fn send(&mut self, data: &[u8]) -> Result<(), TransmissionError> {
        self.crypto
            .encrypt_into(data, &mut self.send_buffer)
            .map_err(TransmissionError::EncryptMessage)?;
        self.connection
            .send(&self.send_buffer)
            .map_err(|e| TransmissionError::Connection(e.to_string()))
    }

    /// receive data and decrypt it directly into the provided buffer, reusing its allocation.
    fn receive_into(&mut self, buffer: &mut Vec<u8>) -> Result<usize, TransmissionError> {
        self.connection
            .receive_into(buffer)
            .map_err(|e| TransmissionError::Connection(e.to_string()))?;
        self.crypto
            .decrypt_in_place(buffer)
            .map_err(TransmissionError::DecryptMessage)?;
        Ok(buffer.len())
    }

    fn shutdown(&self, how: std::net::Shutdown) -> Result<(), Self::ErrorType> {
        self.connection
            .shutdown(how)
            .map_err(|err| TransmissionError::Connection(err.to_string()))
    }

    fn try_clone(&self) -> Result<Self, Self::ErrorType>
    where
        Self: Sized,
    {
        let connection = self
            .connection
            .try_clone()
            .map_err(|err| TransmissionError::Connection(err.to_string()))?;

        Ok(Self {
            connection,
            crypto: self.crypto.clone(),
            send_buffer: Vec::new(),
        })
    }
}