use std::{io::{Result, Read, Error, ErrorKind}, net::TcpStream};
use super::constants::{ HEADER_SIZE_ID, HEADER_SIZE_PACKET_SIZE, HEADER_ID_PACKET };

pub struct PacketAssembler {
  leftover: Option<Vec<u8>>
}

impl PacketAssembler {

  pub fn new() -> PacketAssembler {
    PacketAssembler { leftover: None }
  }

  pub fn assemble(&mut self, tcp_stream: &mut TcpStream) -> Result<Vec<u8>> {
    let mut packet_cursor = 0;

    let mut buffer: Vec<u8>;
    if self.leftover.is_some() {
      buffer = self.leftover.take().unwrap();
    }
    else {
      buffer = Vec::new();
      tcp_stream.read_to_end(&mut buffer).unwrap();
    }

    if !self.is_packet_chunk(&buffer) {
      return Err(Error::new(ErrorKind::InvalidData, "invalid packet chunk header"));
    }

    // create a new packet
    let packet_size = self.get_packet_size(&buffer);
    let mut packet: Vec<u8> = vec![0 as u8; packet_size];
    buffer.drain(..HEADER_SIZE_ID + HEADER_SIZE_PACKET_SIZE);

    while packet_cursor < packet_size {

      if packet_cursor + buffer.len() > packet_size {
        packet.clone_from_slice(&buffer[..packet_size - packet_cursor]);
        self.leftover = Some(buffer[packet_size - packet_cursor..].to_vec());
        break;
      }
      else {
        packet.clone_from_slice(&buffer);
        packet_cursor = packet_cursor + buffer.len();

        if packet_cursor == packet_size {
          break;
        }
      }

      tcp_stream.read_to_end(&mut buffer).unwrap();
    }

    Ok(packet)
  }

  fn get_packet_size(&self, data: &Vec<u8>) -> usize {
    let size = u32::from_le_bytes(
      data[1..5]
        .try_into()
        .expect("could not extract u32 from array size != 4"),
    );
    return size as usize;
  }

  fn is_packet_chunk(&self, data: &Vec<u8>) -> bool {
    return data[0] == HEADER_ID_PACKET;
  }
}
