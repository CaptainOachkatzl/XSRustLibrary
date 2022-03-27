use std::{
  io::Result,
  net::TcpStream,
  u8,
};

use super::{packet_slicer::PacketSlicer, packet_assembler::PacketAssembler};

pub struct TcpPacketConnection {
  tcp_stream: TcpStream,
  packet_slicer: PacketSlicer,
  packet_assembler: PacketAssembler,
}

impl TcpPacketConnection {
  pub fn new(tcp_stream: TcpStream) -> TcpPacketConnection {
    TcpPacketConnection {
      tcp_stream,
      packet_slicer: PacketSlicer::new(1024),
      packet_assembler: PacketAssembler::new(),
    }
  }

  pub fn send(&mut self, data: &[u8]) -> Result<()> {
    self.packet_slicer.slice(&mut self.tcp_stream, data)?;
    Ok(())
  }

  pub fn receive(&mut self) -> Result<Vec<u8>> {  
    return self.packet_assembler.assemble(&mut self.tcp_stream);
  }
}
