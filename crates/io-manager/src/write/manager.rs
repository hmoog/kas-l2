use std::{
    marker::PhantomData,
    sync::{Arc, atomic::AtomicBool},
};

use crate::{
    Storage, WriteCmd,
    utils::{CmdQueue, WorkerHandle},
    write::{WriteConfig, WriteWorker},
};

pub struct WriteManager<K: Storage, W: WriteCmd<K::Namespace>> {
    config: WriteConfig,
    queue: CmdQueue<W>,
    worker: WorkerHandle,
    _marker: PhantomData<K>,
}

impl<K: Storage, W: WriteCmd<K::Namespace>> WriteManager<K, W> {
    pub fn new(store: &Arc<K>, config: &WriteConfig, is_shutdown: &Arc<AtomicBool>) -> Self {
        let queue = CmdQueue::new();
        Self {
            worker: WriteWorker::spawn(config, &queue, store, is_shutdown),
            queue,
            config: config.clone(),
            _marker: PhantomData,
        }
    }

    pub fn submit(&self, write: W) {
        if self.queue.push(write) >= self.config.max_batch_size() && self.worker.is_parked() {
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
