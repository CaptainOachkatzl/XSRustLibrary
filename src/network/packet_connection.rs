use std::{
    io::Write,
    net::{Shutdown, TcpStream},
    u8,
};

use displaydoc::Display;
use thiserror::Error;

use crate::{
    cryptography::{
        encryption::{self, aes256_crypto::Aes256Crypto, Encryption},
        key_exchange::{self, curve25519::Curve25519, KeyExchange},
    },
    packet_assembly,
};

use super::packet_assembly::PacketAssembly;

#[derive(Debug, Display, Error)]
pub enum Error {
    /// IO error: {0}
    IOError(#[from] std::io::Error),
    /// Failed to assemble packet: {0}
    PacketAssembly(#[from] packet_assembly::Error),
    /// Crypto initialization error: {0}
    CryptoInitialization(String),
    /// Failed to encrypt message: {0}
    EncryptMessage(encryption::Error),
}

pub enum KEX {
    Curve25519,
}

pub enum Crypto {
    Aes256,
}

pub struct PacketConnection {
    tcp_stream: TcpStream,
    shutdown_ref_stream: TcpStream,
    packet_assembler: PacketAssembly,
    crypto: Option<Box<dyn Encryption>>,
}

impl PacketConnection {
    pub fn new(tcp_stream: TcpStream, receive_buffer_size: usize) -> PacketConnection {
        PacketConnection {
            // copy stream to have an independently accessible object to shutdown
            // underlying socket guarantees threadsafety
            shutdown_ref_stream: tcp_stream.try_clone().unwrap(),
            tcp_stream,
            packet_assembler: PacketAssembly::new(receive_buffer_size),
            crypto: None,
        }
    }

    pub fn with_encryption(
        tcp_stream: TcpStream,
        receive_buffer_size: usize,
        key_exchange: KEX,
        encryption: Crypto,
        active: bool,
    ) -> Result<Self, Error> {
        let secret = handshake(&tcp_stream, key_exchange, active)?;

        let crypto = Box::from(match encryption {
            Crypto::Aes256 => Aes256Crypto::new(&secret),
        });

        let mut connection = Self::new(tcp_stream, receive_buffer_size);
        connection.crypto = Some(crypto);

        Ok(connection)
    }

    pub fn send(&mut self, packet: &[u8]) -> Result<(), Error> {
        let encrypted = if let Some(ref mut crypto) = self.crypto {
            Some(crypto.encrypt(packet).map_err(Error::EncryptMessage)?)
        } else {
            None
        };

        let packet: &[u8] = match encrypted {
            Some(ref v) => v,
            None => packet,
        };

        self.tcp_stream.write_all(&(packet.len() as u32).to_le_bytes())?; // header
        self.tcp_stream.write_all(packet)?;
        self.tcp_stream.flush()?;
        Ok(())
    }

    pub fn receive(&mut self) -> Result<Vec<u8>, Error> {
        let packet = match self.packet_assembler.receive_packet(&mut self.tcp_stream) {
            Ok(v) => Ok(v),
            Err(e) => {
                self.tcp_stream.shutdown(Shutdown::Both)?;
                Err(Error::PacketAssembly(e))
            }
        }?;

        let packet = match self.crypto {
            Some(ref mut crypto) => crypto.decrypt(&packet).map_err(Error::EncryptMessage)?,
            None => packet,
        };

        Ok(packet)
    }

    pub fn shutdown(&self, how: Shutdown) -> Result<(), Error> {
        self.shutdown_ref_stream.shutdown(how)?;
        Ok(())
    }

    pub fn tcp_stream(&self) -> &TcpStream {
        &self.tcp_stream
    }
}

fn handshake(tcp_stream: &TcpStream, kex: KEX, active: bool) -> Result<[u8; 32], Error> {
    let mut send_con = PacketConnection::new(tcp_stream.try_clone()?, 64);
    let send_kex = move |packet: &[u8]| send_con.send(packet).map_err(|e| key_exchange::Error::Communication(e.to_string()));

    let mut receive_con = PacketConnection::new(tcp_stream.try_clone()?, 64);
    let receive_kex = move || {
        receive_con
            .receive()
            .map(|v| v.into_boxed_slice())
            .map_err(|e| key_exchange::Error::Communication(e.to_string()))
    };

    let secret_result = match kex {
        KEX::Curve25519 => {
            if active {
                Curve25519::handshake_active(send_kex, receive_kex)
            } else {
                Curve25519::handshake_passive(send_kex, receive_kex)
            }
        }
    };

    let secret_data = match secret_result {
        Ok(v) => v,
        Err(e) => return Err(Error::CryptoInitialization(e.to_string())),
    };

    let secret: [u8; 32] = secret_data
        .as_ref()
        .try_into()
        .map_err(|_| Error::CryptoInitialization("Key exchange secret has invalid size.".to_string()))?;

    Ok(secret)
}
