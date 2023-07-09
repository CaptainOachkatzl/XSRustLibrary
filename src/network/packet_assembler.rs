use super::{
    constants::HEADER_SIZE,
    data_buffer::DataBuffer,
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
pub struct PacketAssembler {
    buffer: DataBuffer,
}

impl PacketAssembler {
    pub fn new(buffer_size: usize) -> PacketAssembler {
        PacketAssembler {
            buffer: DataBuffer::new(buffer_size),
        }
    }

    pub fn assemble(&mut self, tcp_stream: &mut TcpStream) -> Result<Vec<u8>, Error> {
        if self.buffer.is_empty() {
            self.receive(tcp_stream)?;
        }

        // create a new packet
        let packet_size = get_packet_size(&mut self.buffer)?;
        let mut packet = PacketBuffer::new(packet_size);

        loop {
            match packet.fill(&mut self.buffer) {
                PacketState::Finished => return Ok(packet.into_vec()),
                PacketState::RequiresData => self.receive(tcp_stream)?,
            }
        }
    }

    fn receive(&mut self, tcp_stream: &mut TcpStream) -> Result<(), Error> {
        let size = tcp_stream.read(&mut self.buffer.get_mut_buffer())?;
        self.buffer.set_positions(0, size);
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
