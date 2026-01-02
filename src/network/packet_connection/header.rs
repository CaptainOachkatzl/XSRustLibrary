use std::io::{Read, Write};

use crate::packet_connection::packet_assembly;

pub const HEADER_SIZE: usize = 4;

pub struct Header {
    packet_size: usize,
}

impl Header {
    pub fn from_packet_data(packet_data: &[u8]) -> Self {
        Self {
            packet_size: packet_data.len(),
        }
    }

    pub fn from_slice(header_data: [u8; HEADER_SIZE]) -> Self {
        let packet_size = u32::from_le_bytes(header_data);
        Self {
            packet_size: packet_size as usize,
        }
    }

    pub fn write(&self, writer: &mut impl Write) -> Result<(), std::io::Error> {
        writer.write_all(&(self.packet_size as u32).to_le_bytes())
    }

    pub fn read(reader: &mut impl Read) -> Result<Self, packet_assembly::Error> {
        let mut buffer = [0; HEADER_SIZE];
        let header_data_size = reader.read(&mut buffer)?;
        if header_data_size != HEADER_SIZE {
            return Err(packet_assembly::Error::InvalidData);
        }
        Ok(Self::from_slice(buffer))
    }

    pub fn packet_size(&self) -> usize {
        self.packet_size
    }
}
