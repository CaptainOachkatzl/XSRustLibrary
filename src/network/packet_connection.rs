mod header;
mod packet_assembly;
pub mod packet_receive_event;

use std::net::{Shutdown, TcpStream};

use displaydoc::Display;
use thiserror::Error;

use crate::connection::Connection;

use packet_assembly::PacketAssembly;

#[derive(Debug, Display, Error)]
pub enum Error {
    /// IO error: {0}
    IOError(#[from] std::io::Error),
    /// Failed to assemble packet: {0}
    PacketAssembly(#[from] packet_assembly::Error),
}

/// TCP connection that sends and receives sized packages instead of streaming data.
pub struct PacketConnection {
    tcp_stream: TcpStream,
    packet_assembler: PacketAssembly,
}

impl PacketConnection {
    pub fn new(tcp_stream: TcpStream, receive_buffer_size: usize) -> PacketConnection {
        PacketConnection {
            tcp_stream,
            packet_assembler: PacketAssembly::new(receive_buffer_size),
        }
    }

    /// get the underlying tcp stream.
    pub fn tcp_stream(&self) -> &TcpStream {
        &self.tcp_stream
    }
}

impl Connection for PacketConnection {
    type ErrorType = Error;

    fn send(&mut self, packet_content: &[u8]) -> Result<(), Error> {
        Ok(PacketAssembly::write_packet(
            &mut self.tcp_stream,
            packet_content,
        )?)
    }

    fn receive(&mut self) -> Result<Vec<u8>, Error> {
        match self.packet_assembler.receive_packet(&mut self.tcp_stream) {
            Ok(v) => Ok(v),
            Err(e) => {
                self.tcp_stream.shutdown(Shutdown::Both)?;
                Err(Error::PacketAssembly(e))
            }
        }
    }

    fn receive_into(&mut self, buffer: &mut Vec<u8>) -> Result<usize, Error> {
        match self
            .packet_assembler
            .receive_packet_into(&mut self.tcp_stream, buffer)
        {
            Ok(size) => Ok(size),
            Err(e) => {
                self.tcp_stream.shutdown(Shutdown::Both)?;
                Err(Error::PacketAssembly(e))
            }
        }
    }

    fn shutdown(&self, how: Shutdown) -> Result<(), Self::ErrorType> {
        self.tcp_stream.shutdown(how)?;
        Ok(())
    }

    fn try_clone(&self) -> Result<Self, Self::ErrorType>
    where
        Self: Sized,
    {
        let tcp_stream = self.tcp_stream.try_clone()?;

        Ok(Self::new(tcp_stream, self.packet_assembler.buffer_size()))
    }
}