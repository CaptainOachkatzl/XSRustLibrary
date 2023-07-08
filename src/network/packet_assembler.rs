use super::constants::HEADER_SIZE;
use std::io::{Error, ErrorKind, Result};

pub struct PacketAssembler {
    leftover: Option<Vec<u8>>,
}

impl Default for PacketAssembler {
    fn default() -> Self {
        Self::new()
    }
}

impl PacketAssembler {
    pub fn new() -> PacketAssembler {
        PacketAssembler { leftover: None }
    }

    pub fn assemble(&mut self, mut receive: impl FnMut() -> Result<Vec<u8>>, shutdown: impl Fn() -> Result<()>) -> Result<Vec<u8>> {
        let mut buffer = match self.leftover.take() {
            Some(v) => v,
            None => receive()?,
        };

        if self.is_fin(&buffer) {
            shutdown()?;
            return Err(Error::new(ErrorKind::InvalidData, "received TCP FIN -> connection closed"));
        }

        // create a new packet
        let packet_size = self.get_packet_size(&mut buffer)?;
        let mut packet: Vec<u8> = vec![0_u8; packet_size];

        self.fill_packet(&mut packet, packet_size, &mut buffer, receive)?;
        Ok(packet)
    }

    fn is_fin(&self, data: &Vec<u8>) -> bool {
        data.is_empty()
    }

    /// read the first 4 bytes of the buffer to determine the packet size
    fn get_packet_size(&self, buffer: &mut Vec<u8>) -> Result<usize> {
        if buffer.len() < HEADER_SIZE {
            return Err(Error::new(ErrorKind::InvalidData, "invalid packet size header"));
        }

        let result: Vec<u8> = buffer.drain(..HEADER_SIZE).collect::<Vec<u8>>();

        let size = u32::from_le_bytes(TryInto::<[u8; 4]>::try_into(result).unwrap());
        Ok(size as usize)
    }

    fn fill_packet(
        &mut self,
        packet: &mut [u8],
        packet_size: usize,
        buffer: &mut Vec<u8>,
        mut receive: impl FnMut() -> Result<Vec<u8>>,
    ) -> Result<()> {
        let mut packet_cursor = 0;
        while packet_cursor < packet_size {
            if !buffer.is_empty() {
                if packet_cursor + buffer.len() > packet_size {
                    // since too much data is availabe for the current packet, fill it up and store the unused data as leftover
                    packet.clone_from_slice(&buffer[..packet_size - packet_cursor]);
                    self.leftover = Some(buffer[packet_size - packet_cursor..].to_vec());
                    break;
                } else {
                    // all the data fits in the current packet
                    packet[packet_cursor..packet_cursor + buffer.len()].clone_from_slice(buffer);
                    packet_cursor += buffer.len();

                    // if the available data is an exact fit for the current packet nothing else needs to be done
                    if packet_cursor == packet_size {
                        break;
                    }
                }
            }
            *buffer = receive()?;
        }

        Ok(())
    }
}
