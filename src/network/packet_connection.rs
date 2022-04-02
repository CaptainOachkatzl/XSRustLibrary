use std::{
  io::{Result, Write, Read},
  net::TcpStream,
  u8,
};

use super::{packet_assembler::PacketAssembler, constants::{HEADER_ID_PACKET}};

pub struct PacketConnection {
  tcp_stream: TcpStream,
  packet_assembler: PacketAssembler,
}

impl PacketConnection {
  pub fn new(tcp_stream: TcpStream) -> PacketConnection {
    PacketConnection {
      tcp_stream,
      packet_assembler: PacketAssembler::new(),
    }
  }

  pub fn send(&mut self, data: &[u8]) -> Result<()> {
    self.write_header(data.len())?;
    self.tcp_stream.write(data)?;
    self.tcp_stream.flush()?;
    Ok(())
  }

  fn write_header(&mut self, length: usize) -> Result<()> {
    let mut header = [0 as u8; 5];
    // indicate data package with first byte 0x00
    header[0] = HEADER_ID_PACKET;
    header[1..5].copy_from_slice(&(length as u32).to_le_bytes());
    self.tcp_stream.write(&header)?;
    Ok(())
  }

  pub fn receive(&mut self) -> Result<Vec<u8>> {  
    let mut receive_call = || {
      let mut buffer = [1 as u8; 256];
      let size = self.tcp_stream.read(&mut buffer)?;

      return Ok(Vec::from(&buffer[..size]));
    };
    return self.packet_assembler.assemble(&mut receive_call);
  }
}
