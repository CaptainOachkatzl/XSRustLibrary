use super::{constants::HEADER_SIZE, data_buffer::DataBuffer};
use displaydoc::Display;
use std::{io::Read, net::TcpStream};
use thiserror::Error;

#[derive(Debug, Display, Error)]
pub enum Error {
    /// Remote sent FIN signal
    ReceivedFin,
    /// Invalid packet data
    InvalidData,
    /// Socket error while trying to receive data
    ReceiveError(#[from] std::io::Error),
}

#[derive(Clone)]
pub struct PacketAssembler {
    data_buffer: DataBuffer,
}

impl PacketAssembler {
    pub fn new(buffer_size: usize) -> PacketAssembler {
        PacketAssembler {
            data_buffer: DataBuffer::new(buffer_size),
        }
    }

    pub fn assemble(&mut self, tcp_stream: &mut TcpStream) -> Result<Vec<u8>, Error> {
        if self.data_buffer.is_empty() {
            receive(tcp_stream, &mut self.data_buffer)?
        }

        if is_fin(&self.data_buffer) {
            return Err(Error::ReceivedFin);
        }

        // create a new packet
        let packet_size = get_packet_size(&mut self.data_buffer)?;
        let mut packet: Vec<u8> = vec![0_u8; packet_size];

        fill_packet(&mut packet, packet_size, &mut self.data_buffer, tcp_stream)?;
        Ok(packet)
    }
}

fn is_fin(data: &DataBuffer) -> bool {
    data.is_empty()
}

/// read the first 4 bytes of the buffer to determine the packet size
fn get_packet_size(data: &mut DataBuffer) -> Result<usize, Error> {
    let header = data.take(HEADER_SIZE);
    if header.len() < HEADER_SIZE {
        return Err(Error::InvalidData);
    }

    let size = u32::from_le_bytes(TryInto::<[u8; 4]>::try_into(header).unwrap());
    Ok(size as usize)
}

fn fill_packet(packet: &mut [u8], packet_size: usize, data: &mut DataBuffer, tcp_stream: &mut TcpStream) -> Result<(), Error> {
    let mut packet_cursor = 0;
    while packet_cursor < packet_size {
        if !data.is_empty() {
            let remaining_packet_space = packet_size - packet_cursor;
            if data.remaining() > remaining_packet_space {
                // since too much data is availabe for the current packet, fill it up and keep unused data in the buffer
                packet.clone_from_slice(data.take(remaining_packet_space));
                break;
            } else {
                // all the data fits in the current packet
                let remaining_data = data.take_to_end();
                packet[packet_cursor..packet_cursor + remaining_data.len()].clone_from_slice(remaining_data);
                packet_cursor += remaining_data.len();

                // if the available data is an exact fit for the current packet nothing else needs to be done
                if packet_cursor == packet_size {
                    break;
                }
            }
        }
        receive(tcp_stream, data)?;
    }

    Ok(())
}

fn receive(tcp_stream: &mut TcpStream, buffer: &mut DataBuffer) -> Result<(), Error> {
    let size = tcp_stream.read(buffer.get_mut_buffer())?;
    buffer.set_positions(0, size);
    Ok(())
}
