use crate::{
    data_buffer::DataBuffer,
    packet_connection::header::{HEADER_SIZE, Header},
};

use super::packet_buffer::{PacketBuffer, PacketState};
use displaydoc::Display;
use std::io::{Cursor, Read, Write};
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

    pub fn write_packet(
        transmission_buffer: &mut impl Write,
        packet_content: &[u8],
    ) -> Result<(), std::io::Error> {
        let header = Header::from_packet_content(packet_content);
        header.write(transmission_buffer)?;
        transmission_buffer.write_all(packet_content)?;
        transmission_buffer.flush()?;
        Ok(())
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
    use std::io::{Cursor, Seek};

    use super::*;

    #[test]
    fn assemble_from_single_byte_chunks() {
        let data = b"0123";
        let mut buffer = Cursor::new(Vec::new());
        PacketAssembly::write_packet(&mut buffer, data).unwrap();
        buffer.rewind().unwrap();
        let assembled_data = PacketAssembly::new(1).receive_packet(&mut buffer).unwrap();
        assert_eq!(&assembled_data, data);
    }

    #[test]
    fn assemble_large_packet() {
        let data = &[1; 1024 * 1024];
        let mut buffer = Cursor::new(Vec::new());
        PacketAssembly::write_packet(&mut buffer, data).unwrap();
        buffer.rewind().unwrap();
        let assembled_data = PacketAssembly::new(1024)
            .receive_packet(&mut buffer)
            .unwrap();
        assert_eq!(&assembled_data, data);
    }

    #[test]
    fn assemble_multiple() {
        let mut buffer = Cursor::new(Vec::new());

        const PACKET_COUNT: usize = 10;
        const PACKET_SIZE: usize = 10;
        for i in 0..PACKET_COUNT {
            let packet_content = &[i as u8; PACKET_SIZE];
            PacketAssembly::write_packet(&mut buffer, &packet_content[..]).unwrap();
        }

        buffer.rewind().unwrap();

        let mut packet_assembly = PacketAssembly::new(HEADER_SIZE + PACKET_SIZE);
        for i in 0..PACKET_COUNT {
            let packet_content = packet_assembly.receive_packet(&mut buffer).unwrap();
            assert_eq!(packet_content, &[i as u8; PACKET_SIZE]);
        }
    }
}
