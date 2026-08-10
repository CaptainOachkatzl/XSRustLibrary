use std::io::Read;

/// low level buffer struct to allow for window views into the data without copying/moving it
#[derive(Clone)]
pub struct DataBuffer {
    buffer: Vec<u8>,
    current_pos: usize,
    end_pos: usize,
}

impl DataBuffer {
    /// creates a new data buffer with the passed capacity.
    /// the buffer is EMPTY and needs to be filled via the `refill` call.
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
        self.rewind();
        Ok(self.end_pos)
    }

    /// rewind the data cursor to the start. this will make previously read data readable again.
    pub fn rewind(&mut self) {
        self.current_pos = 0;
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

    /// read the next <count> bytes and return a slice to them. future reads/takes wont be able to read the taken data again.
    pub fn read_slice(&mut self, count: usize) -> &[u8] {
        let start = self.current_pos;
        let end = std::cmp::min(self.current_pos + count, self.end_pos);
        self.current_pos += count;
        &self.buffer[start..end]
    }

    /// read until the end. future reads/takes wont read any data.
    pub fn read_slice_to_end(&mut self) -> &[u8] {
        let start = self.current_pos;
        self.current_pos = self.end_pos;
        &self.buffer[start..self.end_pos]
    }

    pub fn buffer_size(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    /// get the remaining data that is left in the buffer.
    pub fn remaining(&self) -> usize {
        self.end_pos - self.current_pos
    }

    pub fn current_position(&self) -> usize {
        self.current_pos
    }

    pub fn end_position(&self) -> usize {
        self.end_pos
    }
}

impl Read for DataBuffer {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let data = self.read_slice(buf.len());
        let size = std::cmp::max(data.len(), buf.len());
        buf.copy_from_slice(&data[..size]);
        Ok(size)
    }
}
