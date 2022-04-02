use super::constants::{HEADER_ID_PACKET, HEADER_SIZE_ID, HEADER_SIZE_PACKET_SIZE};
use std::io::{Error, ErrorKind, Result};

pub struct PacketAssembler {
  leftover: Option<Vec<u8>>,
}

impl PacketAssembler {
  pub fn new() -> PacketAssembler {
    PacketAssembler { leftover: None }
  }

  pub fn assemble<'a>(&mut self, receive: &mut dyn FnMut() -> Result<Vec<u8>>) -> Result<Vec<u8>> {
    let mut packet_cursor = 0;

    let mut buffer: Vec<u8>;
    if self.leftover.is_some() {
      buffer = self.leftover.take().unwrap();
    } else {
      buffer = receive()?;
    }

    if !self.is_packet_chunk(&buffer) {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "invalid packet chunk header",
      ));
    }

    // create a new packet
    let packet_size = self.get_packet_size(&buffer)?;
    let mut packet: Vec<u8> = vec![0 as u8; packet_size];
    buffer.drain(..HEADER_SIZE_ID + HEADER_SIZE_PACKET_SIZE);

    while packet_cursor < packet_size {
      if buffer.len() > 0 {
        if packet_cursor + buffer.len() > packet_size {
          packet.clone_from_slice(&buffer[..packet_size - packet_cursor]);
          self.leftover = Some(buffer[packet_size - packet_cursor..].to_vec());
          break;
        } else {
          packet[packet_cursor..packet_cursor + buffer.len()].clone_from_slice(&buffer);
          packet_cursor = packet_cursor + buffer.len();

          if packet_cursor == packet_size {
            break;
          }
        }
      }

      buffer = receive()?;
    }

    Ok(packet)
  }

  fn get_packet_size(&self, data: &Vec<u8>) -> Result<usize> {
    let result = data[1..5].try_into();
    if result.is_err() {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "invalid packet size header",
      ));
    }

    let size = u32::from_le_bytes(result.unwrap());
    return Ok(size as usize);
  }

  fn is_packet_chunk(&self, data: &Vec<u8>) -> bool {
    return data[0] == HEADER_ID_PACKET;
  }
}
