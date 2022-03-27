use std::io::{Result, Write};
use std::net::TcpStream;

const HEADER_SIZE_ID: usize = 1;
const HEADER_SIZE_PACKET_SIZE: usize = 4;

const HEADER_ID_PACKET: u8 = 0x00;

pub struct PacketSlicer {
  max_chunk_size: usize,
}

impl PacketSlicer {
  pub fn new(max_chunk_size: usize) -> PacketSlicer {
    PacketSlicer {
      max_chunk_size,
    }
  }

  pub fn slice(&mut self, tcp_stream: &mut TcpStream, data: &[u8]) -> Result<()> {
    self.write_header(tcp_stream, data.len())?;
    self.write_data(tcp_stream, data)?;
    Ok(())
  }

  fn write_header(&mut self, tcp_stream: &mut TcpStream, length: usize) -> Result<()> {
    // indicate data package with first byte 0x00
    tcp_stream.write(&[HEADER_ID_PACKET; HEADER_SIZE_ID])?;
    tcp_stream.write(&(length as u32).to_le_bytes())?;
    Ok(())
  }

  fn write_data(&mut self, tcp_stream: &mut TcpStream, data: &[u8]) -> Result<()> {
    let mut data_cursor = 0;
    let mut packet_cursor = HEADER_SIZE_ID + HEADER_SIZE_PACKET_SIZE;

    while data_cursor < data.len() {
      let data_end;
      if data.len() - data_cursor <= self.max_chunk_size - packet_cursor {
        data_end = data.len();
      }
      else {
        data_end = data_cursor + self.max_chunk_size - packet_cursor;
      }

      tcp_stream.write(&data[data_cursor..data_end])?;
      tcp_stream.flush()?;
      data_cursor = data_end;
      packet_cursor = 0;
    }

    Ok(())
  }
}
