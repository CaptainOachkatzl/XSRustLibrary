use std::{
  io::{Result, Write, Read},
  net::{TcpStream, Shutdown},
  u8,
};

use super::{packet_assembler::PacketAssembler, constants::{HEADER_ID_PACKET}};

pub struct PacketConnection {
  tcp_stream: TcpStream,
  shutdown_ref_stream: TcpStream,
  packet_assembler: PacketAssembler,
  receive_buffer: Vec<u8>,
}

impl PacketConnection {
  pub fn new(tcp_stream: TcpStream, receive_buffer_size: usize) -> PacketConnection {
    PacketConnection {
      // copy stream to have an independently accessible object to shutdown
      // underlying socket guarantees threadsafety
      shutdown_ref_stream: tcp_stream.try_clone().unwrap(),
      tcp_stream,
      packet_assembler: PacketAssembler::new(),
      receive_buffer: vec![0 as u8; receive_buffer_size],
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
      let size = self.tcp_stream.read(&mut self.receive_buffer)?;
      return Ok(Vec::from(&self.receive_buffer[..size]));
    };

    let shutdown_call = || {
      return self.shutdown_ref_stream.shutdown(Shutdown::Both);
    };

    return self.packet_assembler.assemble(&mut receive_call, &shutdown_call);
  }

  pub fn shutdown(&self, how: Shutdown) -> Result<()> {
    self.shutdown_ref_stream.shutdown(how)?;
    Ok(())
  }

  pub fn try_clone(&self) -> Result<PacketConnection> {
    Ok(PacketConnection { 
      tcp_stream: self.tcp_stream.try_clone()?,
      shutdown_ref_stream: self.tcp_stream.try_clone()?,
      packet_assembler: PacketAssembler::new(), 
      receive_buffer: Vec::clone(&self.receive_buffer)
    })
  }
}
