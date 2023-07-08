#![allow(unused)]

/// low level buffer struct to allow for window views into the data without copying/moving it
#[derive(Clone)]
pub struct DataBuffer {
    buffer: Vec<u8>,
    current_pos: usize,
    end_pos: usize,
}

impl DataBuffer {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            buffer: vec![0_u8; buffer_size],
            current_pos: 0,
            end_pos: 0,
        }
    }

    pub fn set_positions(&mut self, start: usize, end: usize) {
        self.current_pos = start;
        self.end_pos = end;
    }

    pub fn get_mut_buffer(&mut self) -> &mut Vec<u8> {
        &mut self.buffer
    }

    // read the next <count> bytes. future reads/takes will be able to read the data again.
    pub fn read(&self, count: usize) -> &[u8] {
        let start = self.current_pos;
        let end = std::cmp::min(self.current_pos + count, self.end_pos);
        &self.buffer[start..end]
    }

    /// read until the end. future reads/takes will still read this data.
    pub fn read_to_end(&self) -> &[u8] {
        &self.buffer[self.current_pos..self.end_pos]
    }

    /// read the next <count> bytes. future reads/takes wont be able to read the taken data again.
    pub fn take(&mut self, count: usize) -> &[u8] {
        let start = self.current_pos;
        let end = std::cmp::min(self.current_pos + count, self.end_pos);
        self.current_pos += count;
        &self.buffer[start..end]
    }

    /// read until the end. future reads/takes wont read any data.
    pub fn take_to_end(&mut self) -> &[u8] {
        let start = self.current_pos;
        self.current_pos = self.end_pos;
        &self.buffer[start..self.end_pos]
    }

    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    pub fn remaining(&self) -> usize {
        self.end_pos - self.current_pos
    }

    pub fn get_current_position(&self) -> usize {
        self.current_pos
    }

    pub fn get_end_position(&self) -> usize {
        self.end_pos
    }
}
