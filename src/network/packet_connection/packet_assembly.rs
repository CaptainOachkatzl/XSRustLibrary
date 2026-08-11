use crate::packet_connection::header::{HEADER_SIZE, Header};

use displaydoc::Display;
use std::io::{Read, Write};
use thiserror::Error;

#[derive(Debug, Display, Error)]
pub enum Error {
    /// Remote sent FIN signal before packet assembly was complete
    ReceivedFin,
    /// IO error while trying to receive data: {0}
    Receive(#[from] std::io::Error),
    /// Remote sent a packet above the maximum size
    OversizedPacket,
}

#[derive(Clone)]
pub struct PacketAssembly {
    max_packet_size: usize,
}

impl PacketAssembly {
    pub fn new() -> Self {
        Self::with_max_packet_size(usize::MAX)
    }

    pub fn with_max_packet_size(max_packet_size: usize) -> Self {
        Self { max_packet_size }
    }

    pub fn max_packet_size(&self) -> usize {
        self.max_packet_size
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
        let mut output = Vec::new();
        self.receive_packet_into(data, &mut output)?;
        Ok(output)
    }

    /// receive a packet and write its content directly into the provided buffer,
    /// reusing the buffer's allocation.
    pub fn receive_packet_into(
        &mut self,
        data: &mut impl Read,
        buffer: &mut Vec<u8>,
    ) -> Result<usize, Error> {
        let mut header_data = [0_u8; HEADER_SIZE];
        self.assemble_section_into(data, &mut header_data)?;
        let header = Header::from_slice(header_data);
        let packet_size = header.packet_size();

        if packet_size > self.max_packet_size {
            return Err(Error::OversizedPacket);
        }

        buffer.clear();
        buffer.resize(packet_size, 0);
        self.assemble_section_into(data, buffer)?;

        Ok(packet_size)
    }

    /// pull data from the stream until the provided section is completely filled.
    fn assemble_section_into(
        &mut self,
        data: &mut impl Read,
        buffer: &mut [u8],
    ) -> Result<(), Error> {
        data.read_exact(buffer)?;
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
        let assembled_data = PacketAssembly::new().receive_packet(&mut buffer).unwrap();
        assert_eq!(&assembled_data, data);
    }

    #[test]
    fn assemble_large_packet() {
        let data = &[1; 1024 * 1024];
        let mut buffer = Cursor::new(Vec::new());
        PacketAssembly::write_packet(&mut buffer, data).unwrap();
        buffer.rewind().unwrap();
        let assembled_data = PacketAssembly::new().receive_packet(&mut buffer).unwrap();
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

        let mut packet_assembly = PacketAssembly::new();
        for i in 0..PACKET_COUNT {
            let packet_content = packet_assembly.receive_packet(&mut buffer).unwrap();
            assert_eq!(packet_content, &[i as u8; PACKET_SIZE]);
        }
    }

    #[test]
    fn assemble_different_sizes() {
        let mut rng = fastrand::Rng::with_seed(0);
        let mut buffer = Cursor::new(Vec::new());

        const PACKET_COUNT: usize = 10;

        let mut packet_sizes = Vec::new();

        for i in 0..PACKET_COUNT {
            let packet_size = rng.usize(50..100);
            packet_sizes.push(packet_size);
            let packet_content = vec![i as u8; packet_size];
            PacketAssembly::write_packet(&mut buffer, &packet_content).unwrap();
        }

        buffer.rewind().unwrap();

        let mut packet_assembly = PacketAssembly::new();
        for i in 0..PACKET_COUNT {
            let packet_content = packet_assembly.receive_packet(&mut buffer).unwrap();
            assert_eq!(packet_content, vec![i as u8; packet_sizes[i]]);
        }
    }

    #[test]
    fn receive_packet_into_reuses_buffer() {
        let data = b"hello world";
        let mut buffer = Cursor::new(Vec::new());
        PacketAssembly::write_packet(&mut buffer, data).unwrap();
        buffer.rewind().unwrap();

        let mut packet_assembly = PacketAssembly::new();
        let mut output = vec![0_u8; 1024]; // pre-allocated buffer
        let size = packet_assembly
            .receive_packet_into(&mut buffer, &mut output)
            .unwrap();
        assert_eq!(size, data.len());
        assert_eq!(&output[..size], data);
    }
}
