use super::{constants::HEADER_SIZE, packet_buffer::DataBuffer};
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
    ReceiveError,
}

#[derive(Clone)]
pub struct PacketAssembler {
    receive_buffer: Vec<u8>,
    leftover_position: Option<(usize, usize)>,
}

impl PacketAssembler {
    pub fn new(buffer_size: usize) -> PacketAssembler {
        PacketAssembler {
            receive_buffer: vec![0_u8; buffer_size],
            leftover_position: None,
        }
    }

    pub fn assemble(&mut self, tcp_stream: &mut TcpStream) -> Result<Vec<u8>, Error> {
        let mut data = if let Some((start, end)) = self.leftover_position {
            DataBuffer::with_starting_position(&self.receive_buffer[..end], start)
        } else {
            receive(tcp_stream, &mut self.receive_buffer)?
        };

        if is_fin(&data) {
            return Err(Error::ReceivedFin);
        }

        // create a new packet
        let packet_size = get_packet_size(&mut data)?;
        let mut packet: Vec<u8> = vec![0_u8; packet_size];

        self.leftover_position = fill_packet(
            &mut packet,
            packet_size,
            data.get_current_position(),
            data.get_end_position(),
            tcp_stream,
            &mut self.receive_buffer,
        )?;
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

fn fill_packet<'a, 'b>(
    packet: &mut [u8],
    packet_size: usize,
    current_pos: usize,
    end_pos: usize,
    tcp_stream: &'b mut TcpStream,
    buffer: &'a mut Vec<u8>,
) -> Result<Option<(usize, usize)>, Error> {
    let mut packet_cursor = 0;
    let mut data = DataBuffer::with_starting_position(&buffer[..end_pos], current_pos);
    while packet_cursor < packet_size {
        if !data.is_empty() {
            if packet_cursor + data.remaining() > packet_size {
                // since too much data is availabe for the current packet, fill it up and store the unused data as leftover
                packet.clone_from_slice(&data.take(packet_size - packet_cursor));
                return Ok(Some((data.get_current_position(), data.get_end_position())));
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
        data = receive(tcp_stream, buffer)?;
    }

    Ok(None)
}

fn receive<'a, 'b>(tcp_stream: &'b mut TcpStream, buffer: &'a mut Vec<u8>) -> Result<DataBuffer<'a>, Error> {
    let size = tcp_stream.read(buffer).map_err(|_| Error::InvalidData)?;
    Ok(DataBuffer::new(&buffer[..size]))
}
