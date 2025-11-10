use kas_l2_storage_common_types::Store;

use crate::{read::ReadConfig, write::WriteConfig};

#[derive(Clone, Debug)]
pub struct StorageConfig<S: Store> {
    pub(crate) store: Option<S>,
    pub(crate) write_config: WriteConfig,
    pub(crate) read_config: ReadConfig,
}

impl<S: Store> StorageConfig<S> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_store(mut self, store: S) -> Self {
        self.store = Some(store);
        self
    }

    pub fn with_write_config(mut self, write_config: WriteConfig) -> Self {
        self.write_config = write_config;
        self
    }

    pub fn with_read_config(mut self, read_config: ReadConfig) -> Self {
        self.read_config = read_config;
        self
    }

    pub fn unpack(mut self) -> (S, WriteConfig, ReadConfig) {
        (
            self.store.take().expect("unpack requires store to be set"),
            self.write_config,
            self.read_config,
        )
    }
}

impl<S: Store> Default for StorageConfig<S> {
    fn default() -> Self {
        Self {
            store: None,
            write_config: WriteConfig::default(),
            read_config: ReadConfig::default(),
        }
    }
}
