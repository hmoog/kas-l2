use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use kas_l2_io_core::KVStore;
use kas_l2_runtime_macros::smart_pointer;
use crate::{read, write};

#[smart_pointer]
pub struct IoManager<
    Store: KVStore,
    ReadCmd: read::Cmd<<Store as KVStore>::Namespace>,
    WriteCmd: write::Cmd<<Store as KVStore>::Namespace>,
> {
    reader: read::Manager<Store, ReadCmd>,
    writer: write::Manager<Store, WriteCmd>,
    shutdown_flag: Arc<AtomicBool>,
}

impl<S: KVStore, R: read::Cmd<<S as KVStore>::Namespace>, W: write::Cmd<<S as KVStore>::Namespace>>
    IoManager<S, R, W>
{
    pub fn new(store: S) -> Self {
        let store = Arc::new(store);
        let shutdown_flag = Arc::new(AtomicBool::new(false));

        Self(Arc::new(IoManagerData {
            reader: read::Manager::new(store.clone(), shutdown_flag.clone()),
            writer: write::Manager::new(store, shutdown_flag.clone()),
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
