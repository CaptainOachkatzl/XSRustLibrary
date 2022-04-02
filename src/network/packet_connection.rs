use std::{
  io::{Result, Write},
  net::TcpStream,
  u8,
};

use super::{packet_assembler::PacketAssembler, constants::{HEADER_ID_PACKET, HEADER_SIZE_ID}};

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
    // indicate data package with first byte 0x00
    self.tcp_stream.write(&[HEADER_ID_PACKET; HEADER_SIZE_ID])?;
    self.tcp_stream.write(&(length as u32).to_le_bytes())?;
    Ok(())
  }

  pub fn receive(&mut self) -> Result<Vec<u8>> {  
    return self.packet_assembler.assemble(&mut self.tcp_stream);
  }
}
