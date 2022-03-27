use std::{
  io::{Read, Result},
  net::TcpStream,
  u8,
};

use super::packet_slicer::PacketSlicer;

const HEADER_SIZE_ID: usize = 1;
const HEADER_SIZE_PACKET_SIZE: usize = 4;

const HEADER_ID_PACKET: u8 = 0x00;

pub struct TcpPacketConnection {
  tcp_stream: TcpStream,
  packet_slicer: PacketSlicer,
}

impl TcpPacketConnection {
  pub fn new(tcp_stream: TcpStream) -> TcpPacketConnection {
    TcpPacketConnection {
      tcp_stream,
      packet_slicer: PacketSlicer::new(1024),
    }
  }

  pub fn send(&mut self, data: &[u8]) -> Result<()> {
    self.packet_slicer.slice(&mut self.tcp_stream, data)?;
    Ok(())
  }

  pub fn receive(&mut self) -> Result<Vec<u8>> {
    
    let mut buffer: Vec<u8> = Vec::new();
    let size = self.tcp_stream.read_to_end(&mut buffer).unwrap();

    // create a new packet
    let packet_size = self.get_packet_size(&buffer);
    let mut packet: Vec<u8> = vec![0 as u8; packet_size];

    if size != packet_size + HEADER_SIZE_ID + HEADER_SIZE_PACKET_SIZE {
      panic!("packet size is not compatible with incoming data");
    }

    packet.clone_from_slice(&buffer[5..size]);

    Ok(packet)
  }

  fn get_packet_size(&mut self, data: &Vec<u8>) -> usize {
    let size = u32::from_le_bytes(
      data[1..5]
        .try_into()
        .expect("could not extract u32 from array size != 4"),
    );
    return size as usize;
  }
}
