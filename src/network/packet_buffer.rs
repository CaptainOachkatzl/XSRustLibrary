pub struct DataBuffer<'a> {
    data: &'a [u8],
    current_pos: usize,
}

impl<'a> DataBuffer<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self::with_starting_position(data, 0)
    }

    pub fn with_starting_position(data: &'a [u8], starting_position: usize) -> Self {
        Self {
            data,
            current_pos: starting_position,
        }
    }

    pub fn read<'b>(&'b self, count: usize) -> &'a [u8] {
        let start = self.current_pos;
        let end = std::cmp::min(self.current_pos + count, self.data.len());
        &self.data[start..end]
    }

    pub fn read_to_end<'b>(&'b self) -> &'a [u8] {
        &self.data[self.current_pos..]
    }

    pub fn take(&mut self, count: usize) -> &[u8] {
        let ret = self.read(count);
        self.current_pos += count;
        ret
    }

    pub fn take_to_end(&mut self) -> &[u8] {
        let ret = self.read_to_end();
        self.current_pos = self.data.len();
        ret
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.remaining() <= 0
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.data.len() - self.current_pos
    }

    pub fn get_current_position(&self) -> usize {
        self.current_pos
    }

    pub fn get_end_position(&self) -> usize {
        self.data.len()
    }
}
