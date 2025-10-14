use crate::{read::config::ReadConfig, write::config::WriteConfig};

pub const BATCH_SIZE: usize = 1000;
pub const MAX_READERS: usize = 8;
pub const BUFFER_DEPTH_PER_READER: usize = 128;

#[derive(Default, Clone, Debug)]
pub struct Config {
    write_config: WriteConfig,
    read_config: ReadConfig,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_write_config(mut self, write_config: WriteConfig) -> Self {
        self.write_config = write_config;
        self
    }

    pub fn with_read_config(mut self, read_config: ReadConfig) -> Self {
        self.read_config = read_config;
        self
    }

    pub fn write_config(&self) -> &WriteConfig {
        &self.write_config
    }

    pub fn read_config(&self) -> &ReadConfig {
        &self.read_config
    }
}
