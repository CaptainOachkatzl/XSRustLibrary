use std::{net::{TcpStream}, u8, io::{Result, Write, Read}};

pub struct TcpPacketConnection {
  tcp_stream: TcpStream
}

impl TcpPacketConnection {
    pub fn new(tcp_stream: TcpStream) -> TcpPacketConnection {
      TcpPacketConnection { tcp_stream }
    }

    pub fn send(&mut self, data: &[u8]) -> Result<()> {
      self.tcp_stream.write(data)?;
      self.tcp_stream.flush()?;
      Ok(())
    }

    pub fn receive(&mut self) -> Result<Vec<u8>> {
      let mut buffer: [u8; 256] = [0 as u8; 256];
      let size = self.tcp_stream.read(&mut buffer).unwrap();
      let mut data: Vec<u8> = vec![0; size];
      data.clone_from_slice(&buffer[0..size]);
      Ok(data)
    }
}