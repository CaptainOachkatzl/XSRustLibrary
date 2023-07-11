use crate::data_buffer::DataBuffer;

pub enum PacketState {
    Finished,
    RequiresData,
}

pub struct PacketBuffer {
    buffer: Vec<u8>,
    current_pos: usize,
}

impl PacketBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0_u8; capacity],
            current_pos: 0,
        }
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.buffer
    }

    fn packet_size(&self) -> usize {
        self.buffer.len()
    }

    fn remaining_space(&self) -> usize {
        self.packet_size() - self.current_pos
    }

    pub fn fill(&mut self, data: &mut DataBuffer) -> PacketState {
        if self.remaining_space() == 0 {
            return PacketState::Finished;
        }

        let remaining_space = self.remaining_space();
        if data.remaining() > remaining_space {
            // since too much data is availabe for the current packet, fill it up and keep unused data in the buffer
            self.buffer.clone_from_slice(data.take(remaining_space));
        } else {
            // all the data fits in the current packet
            let remaining_data = data.take_to_end();
            self.buffer[self.current_pos..self.current_pos + remaining_data.len()].clone_from_slice(remaining_data);
            self.current_pos += remaining_data.len();

            if self.remaining_space() > 0 {
                return PacketState::RequiresData;
            }
        }

        PacketState::Finished
    }
}
