#![allow(unused)]

use std::{
    io::{BufReader, Read},
    slice::Windows,
};

/// low level buffer struct to allow for window views into the data without copying/moving it
#[derive(Clone)]
pub struct DataBuffer {
    buffer: Vec<u8>,
    current_pos: usize,
    end_pos: usize,
}

impl DataBuffer {
    /// creates a new data buffer with the passed capacity.
    /// the buffer is EMPTY and needs to be filled via the `get_mut_buffer` call.
    pub fn new(buffer_size: usize) -> Self {
        Self {
            buffer: vec![0_u8; buffer_size],
            current_pos: 0,
            end_pos: 0,
        }
    }

    /// refill the internal buffer with any data source that implements the `Read` trait.
    pub fn refill(&mut self, data: &mut impl Read) -> Result<usize, std::io::Error> {
        self.end_pos = data.read(&mut self.buffer)?;
        self.current_pos = 0;
        Ok(self.end_pos)
    }

    /// read the next <count> bytes. future reads/takes will be able to read the data again.
    pub fn peek(&self, count: usize) -> &[u8] {
        let start = self.current_pos;
        let end = std::cmp::min(self.current_pos + count, self.end_pos);
        &self.buffer[start..end]
    }

    /// read until the end. future reads/takes will still read this data.
    pub fn peek_to_end(&self) -> &[u8] {
        &self.buffer[self.current_pos..self.end_pos]
    }

    /// read the next <count> bytes. future reads/takes wont be able to read the taken data again.
    pub fn read(&mut self, count: usize) -> &[u8] {
        let start = self.current_pos;
        let end = std::cmp::min(self.current_pos + count, self.end_pos);
        self.current_pos += count;
        &self.buffer[start..end]
    }

    /// read until the end. future reads/takes wont read any data.
    pub fn read_to_end(&mut self) -> &[u8] {
        let start = self.current_pos;
        self.current_pos = self.end_pos;
        &self.buffer[start..self.end_pos]
    }

    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    /// get the remaining data that is left in the buffer.
    pub fn remaining(&self) -> usize {
        self.end_pos - self.current_pos
    }

    fn current_position(&self) -> usize {
        self.current_pos
    }

    fn end_position(&self) -> usize {
        self.end_pos
    }
}
