#![allow(unused)]

use std::{io::BufReader, slice::Windows};

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

    /// refill the internal buffer with data. the `refill_internal_buffer` closure needs to return how many bytes of the buffer were filled.
    pub fn refill<E>(&mut self, refill_internal_buffer: impl FnOnce(&mut [u8]) -> Result<usize, E>) -> Result<(), E> {
        self.end_pos = refill_internal_buffer(&mut self.buffer)?;
        self.current_pos = 0;
        Ok(())
    }

    /// read the next <count> bytes. future reads/takes will be able to read the data again.
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

    /// get the remaining data that is left in the buffer.
    pub fn remaining(&self) -> usize {
        self.end_pos - self.current_pos
    }

    fn get_current_position(&self) -> usize {
        self.current_pos
    }

    fn get_end_position(&self) -> usize {
        self.end_pos
    }
}
