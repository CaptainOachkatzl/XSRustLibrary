use std::io::Write;

pub const HEADER_SIZE: usize = 8;

pub struct Header {
    packet_size: usize,
}

impl Header {
    pub fn from_packet_content(packet_data: &[u8]) -> Self {
        Self {
            packet_size: packet_data.len(),
        }
    }

    pub fn from_slice(header_data: [u8; HEADER_SIZE]) -> Self {
        let packet_size = u64::from_le_bytes(header_data);
        Self {
            packet_size: packet_size as usize,
        }
    }

    pub fn write(&self, writer: &mut impl Write) -> Result<(), std::io::Error> {
        writer.write_all(&(self.packet_size as u64).to_le_bytes())
    }

    pub fn packet_size(&self) -> usize {
        self.packet_size
    }
}