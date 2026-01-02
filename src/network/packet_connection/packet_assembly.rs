use crate::{
    data_buffer::DataBuffer,
    packet_connection::header::{HEADER_SIZE, Header},
};

use super::packet_buffer::{PacketBuffer, PacketState};
use displaydoc::Display;
use std::io::{Cursor, Read};
use thiserror::Error;

#[derive(Debug, Display, Error)]
pub enum Error {
    /// Remote sent FIN signal before packet assembly was complete
    ReceivedFin,
    /// Invalid packet data
    InvalidData,
    /// IO error while trying to receive data: {0}
    Receive(#[from] std::io::Error),
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

    pub fn receive_packet(&mut self, data: &mut impl Read) -> Result<Vec<u8>, Error> {
        let header = self.assemble_header(data)?;
        self.assemble_section(data, header.packet_size())
    }

    fn assemble_header(&mut self, data: &mut impl Read) -> Result<Header, Error> {
        let header_data = self.assemble_section(data, HEADER_SIZE)?;
        Header::read(&mut Cursor::new(header_data))
    }

    /// pull data from stream until there is enough data for the section available
    fn assemble_section(
        &mut self,
        data: &mut impl Read,
        section_size: usize,
    ) -> Result<Vec<u8>, Error> {
        let mut section_data = PacketBuffer::new(section_size);

        while section_data.fill(&mut self.buffer) == PacketState::RequiresData {
            self.receive_next_chunk(data)?;
        }

        Ok(section_data.into_vec())
    }

    fn receive_next_chunk(&mut self, data: &mut impl Read) -> Result<(), Error> {
        if self.buffer.refill(data)? == 0 {
            return Err(Error::ReceivedFin);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn assemble_from_single_byte_chunks() {
        let data = b"0123";
        let assembled_data = assemble_packet_with_local_buffer(PacketAssembly::new(1), data);
        assert_eq!(&assembled_data, data);
    }

    #[test]
    fn assemble_large_packet() {
        let data = &[1; 1024 * 1024];
        let assembled_data = assemble_packet_with_local_buffer(PacketAssembly::new(1024), data);
        assert_eq!(&assembled_data, data);
    }

    /// a local buffer is emulating a transmitted package
    fn assemble_packet_with_local_buffer(
        mut packet_assembly: PacketAssembly,
        packet_data: &[u8],
    ) -> Vec<u8> {
        let header = Header::from_packet_content(packet_data);
        let mut send_packet = Vec::with_capacity(HEADER_SIZE + packet_data.len());
        header.write(&mut send_packet).unwrap();
        send_packet.extend_from_slice(packet_data);
        let mut data_reader = Cursor::new(send_packet);
        packet_assembly.receive_packet(&mut data_reader).unwrap()
    }
}
