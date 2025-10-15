use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use kas_l2_runtime_macros::smart_pointer;

use crate::{IoConfig, ReadCmd, Storage, WriteCmd, read::ReadManager, write::WriteManager};

#[smart_pointer]
pub struct IoManager<K: Storage, R: ReadCmd<K::StateSpace>, W: WriteCmd<K::StateSpace>> {
    reader: ReadManager<K, R>,
    writer: WriteManager<K, W>,
    shutdown_flag: Arc<AtomicBool>,
}

impl<K: Storage, R: ReadCmd<K::StateSpace>, W: WriteCmd<K::StateSpace>> IoManager<K, R, W> {
    pub fn new(store: K, config: IoConfig) -> Self {
        let store = Arc::new(store);
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        Self(Arc::new(IoManagerData {
            reader: ReadManager::new(config.read_config(), &store, &shutdown_flag),
            writer: WriteManager::new(&store, config.write_config(), &shutdown_flag),
            shutdown_flag,
        }))
    }

    pub fn submit_read(&self, cmd: R) {
        self.reader.submit(cmd);
    }

    pub fn submit_write(&self, cmd: W) {
        self.writer.submit(cmd);
    }

    pub fn shutdown(&self) {
        self.shutdown_flag.store(true, Ordering::Release);
        self.reader.shutdown();
        self.writer.shutdown();
    }
}
