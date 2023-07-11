use crate::data_buffer::DataBuffer;

use super::{
    constants::HEADER_SIZE,
    packet_buffer::{PacketBuffer, PacketState},
};
use displaydoc::Display;
use std::{io::Read, net::TcpStream};
use thiserror::Error;

#[derive(Debug, Display, Error)]
pub enum Error {
    /// Remote sent FIN signal
    ReceivedFin,
    /// Invalid packet data
    InvalidData,
    /// Socket error while trying to receive data
    ReceiveError(#[from] std::io::Error),
}

#[derive(Clone)]
pub struct PacketAssembly {
    buffer: DataBuffer,
}

impl PacketAssembly {
    pub fn new(buffer_size: usize) -> PacketAssembly {
        PacketAssembly {
            buffer: DataBuffer::new(buffer_size),
        }
    }

    pub fn receive_packet(&mut self, tcp_stream: &mut TcpStream) -> Result<Vec<u8>, Error> {
        if self.buffer.is_empty() {
            self.receive_next_packet_chunk(tcp_stream)?;
        }

        // create a new packet
        let packet_size = get_packet_size(&mut self.buffer)?;
        let mut packet = PacketBuffer::new(packet_size);

        loop {
            match packet.fill(&mut self.buffer) {
                PacketState::Finished => return Ok(packet.into_vec()),
                PacketState::RequiresData => self.receive_next_packet_chunk(tcp_stream)?,
            }
        }
    }

    fn receive_next_packet_chunk(&mut self, tcp_stream: &mut TcpStream) -> Result<(), Error> {
        let size = tcp_stream.read(self.buffer.get_mut_buffer())?;
        self.buffer.reset_read_window(size);
        if size == 0 {
            return Err(Error::ReceivedFin);
        }

        Ok(())
    }
}

/// read the first 4 bytes of the buffer to determine the packet size
fn get_packet_size(data: &mut DataBuffer) -> Result<usize, Error> {
    let header = data.take(HEADER_SIZE);
    if header.len() < HEADER_SIZE {
        return Err(Error::InvalidData);
    }

    let size = u32::from_le_bytes(TryInto::<[u8; 4]>::try_into(header).unwrap());
    Ok(size as usize)
}
