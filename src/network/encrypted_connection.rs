use displaydoc::Display;
use thiserror::Error;

use crate::{
    connection::Connection,
    cryptography::{
        encryption::{self, Encryption},
        key_exchange::{HandshakeMode, KeyExchange},
    },
    packet_connection::PacketConnection,
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

pub struct EncryptedConnection<Enc>
where
    Enc: Encryption,
{
    crypto: Enc,
    connection: PacketConnection,
}

impl<Enc> EncryptedConnection<Enc>
where
    Enc: Encryption,
{
    pub fn with_handshake(mut connection: PacketConnection, kex: impl KeyExchange, mode: HandshakeMode) -> Result<Self, Error> {
        let secret = Self::handshake(&mut connection, kex, mode)?;

        let crypto = Enc::initialize(&secret).map_err(|e| Error::CryptoInitialization(e.to_string()))?;
        Ok(Self {
            connection,
            crypto: *crypto,
        })
    }

    fn handshake(connection: &mut PacketConnection, mut kex: impl KeyExchange, mode: HandshakeMode) -> Result<[u8; 32], Error> {
        let secret_result = kex.handshake(connection, mode);

        let secret_data = match secret_result {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::CryptoInitialization(e.to_string())),
        }?;

        let secret: [u8; 32] = secret_data
            .as_ref()
            .try_into()
            .map_err(|_| Error::CryptoInitialization("Invalid secret size".to_string()))?;

        Ok(secret)
    }
}

impl<Enc> Connection<Error> for EncryptedConnection<Enc>
where
    Enc: Encryption,
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
