use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use kas_l2_io_core::KVStore;
use kas_l2_runtime_macros::smart_pointer;

use crate::{ReadCmd, WriteCmd, read_manager::ReadManager, write_manager::WriteManager};

#[smart_pointer]
pub struct IoManager<K: KVStore, R: ReadCmd<K::Namespace>, W: WriteCmd<K::Namespace>> {
    reader: ReadManager<K, R>,
    writer: WriteManager<K, W>,
    shutdown_flag: Arc<AtomicBool>,
}

impl<K: KVStore, R: ReadCmd<K::Namespace>, W: WriteCmd<K::Namespace>> IoManager<K, R, W> {
    pub fn new(store: K) -> Self {
        let store = Arc::new(store);
        let shutdown_flag = Arc::new(AtomicBool::new(false));

        Self(Arc::new(IoManagerData {
            reader: ReadManager::new(store.clone(), shutdown_flag.clone()),
            writer: WriteManager::new(store, shutdown_flag.clone()),
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
