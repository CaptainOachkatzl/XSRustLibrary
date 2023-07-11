use std::{
    io::Write,
    net::{Shutdown, TcpStream},
    u8,
};

use displaydoc::Display;
use thiserror::Error;

use crate::{connection::Connection, packet_assembly};

use super::packet_assembly::PacketAssembly;

#[derive(Debug, Display, Error)]
pub enum Error {
    /// IO error: {0}
    IOError(#[from] std::io::Error),
    /// Failed to assemble packet: {0}
    PacketAssembly(#[from] packet_assembly::Error),
}

pub struct PacketConnection {
    tcp_stream: TcpStream,
    shutdown_ref_stream: TcpStream,
    packet_assembler: PacketAssembly,
}

impl PacketConnection {
    pub fn new(tcp_stream: TcpStream, receive_buffer_size: usize) -> PacketConnection {
        PacketConnection {
            // copy stream to have an independently accessible object to shutdown
            // underlying socket guarantees threadsafety
            shutdown_ref_stream: tcp_stream.try_clone().unwrap(),
            tcp_stream,
            packet_assembler: PacketAssembly::new(receive_buffer_size),
        }
    }

    pub fn shutdown(&self, how: Shutdown) -> Result<(), Error> {
        self.shutdown_ref_stream.shutdown(how)?;
        Ok(())
    }

    pub fn tcp_stream(&self) -> &TcpStream {
        &self.tcp_stream
    }
}

impl Connection for PacketConnection {
    type ErrorType = Error;

    fn send(&mut self, packet: &[u8]) -> Result<(), Error> {
        self.tcp_stream.write_all(&(packet.len() as u32).to_le_bytes())?; // header
        self.tcp_stream.write_all(packet)?;
        self.tcp_stream.flush()?;
        Ok(())
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
}
