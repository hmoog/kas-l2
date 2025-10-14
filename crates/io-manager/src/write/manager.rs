use std::{
    marker::PhantomData,
    sync::{Arc, atomic::AtomicBool},
};

use kas_l2_io_core::KVStore;

use crate::{
    WriteCmd, cmd_queue::CmdQueue, config::BATCH_SIZE, worker_handle::WorkerHandle,
    write::worker::WriteWorker,
};

pub struct WriteManager<K: KVStore, W: WriteCmd<K::Namespace>> {
    queue: CmdQueue<W>,
    worker: WorkerHandle,
    _marker: PhantomData<K>,
}

impl<K: KVStore, W: WriteCmd<K::Namespace>> WriteManager<K, W> {
    pub fn new(store: &Arc<K>, is_shutdown: &Arc<AtomicBool>) -> Self {
        let queue = CmdQueue::new();
        Self {
            worker: WriteWorker::spawn(&queue, store, is_shutdown),
            queue,
            _marker: PhantomData,
        }
    }

    pub fn submit(&self, write: W) {
        if self.queue.push(write) >= BATCH_SIZE && self.worker.is_parked() {
            self.worker.wake();
        }
    }

    pub fn shutdown(&self) {
        self.worker.wake();

        if let Some(handle) = self.worker.take_join() {
            handle.join().expect("write worker panicked");
        }
    }
}
