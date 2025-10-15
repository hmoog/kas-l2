use crate::{read::ReadConfig, write::WriteConfig};

#[derive(Default, Clone, Debug)]
pub struct StorageConfig {
    write_config: WriteConfig,
    read_config: ReadConfig,
}

impl StorageConfig {
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
