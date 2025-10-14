#[derive(Clone, Debug)]
pub struct ReadConfig {
    max_readers: usize,
    buffer_depth_per_reader: usize,
}

impl ReadConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_readers(mut self, max_readers: usize) -> Self {
        self.max_readers = max_readers;
        self
    }

    pub fn with_buffer_depth_per_reader(mut self, buffer_depth_per_reader: usize) -> Self {
        self.buffer_depth_per_reader = buffer_depth_per_reader;
        self
    }

    pub fn max_readers(&self) -> usize {
        self.max_readers
    }

    pub fn buffer_depth_per_reader(&self) -> usize {
        self.buffer_depth_per_reader
    }
}

impl Default for ReadConfig {
    fn default() -> Self {
        Self {
            max_readers: 8,
            buffer_depth_per_reader: 128,
        }
    }
}
