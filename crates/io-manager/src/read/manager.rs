use std::{
    array,
    marker::PhantomData,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use crossbeam_utils::CachePadded;
use kas_l2_io_core::KVStore;

use crate::{
    ReadCmd,
    cmd_queue::CmdQueue,
    config::{BUFFER_DEPTH_PER_READER, MAX_READERS},
    read::worker::ReadWorker,
    worker_handle::WorkerHandle,
};

pub struct ReadManager<K: KVStore, R: ReadCmd<K::Namespace>> {
    queue: CmdQueue<R>,
    active_readers: Arc<CachePadded<AtomicUsize>>,
    worker_handles: [WorkerHandle; MAX_READERS],
    _marker: PhantomData<K>,
}

impl<K: KVStore, R: ReadCmd<K::Namespace>> ReadManager<K, R> {
    pub fn new(store: &Arc<K>, is_shutdown: &Arc<AtomicBool>) -> Self {
        let queue = CmdQueue::new();
        let active_readers = Arc::new(CachePadded::new(AtomicUsize::new(0)));
        Self {
            worker_handles: array::from_fn(|i| {
                ReadWorker::spawn(i, &queue, store, &active_readers, is_shutdown)
            }),
            queue,
            active_readers,
            _marker: PhantomData,
        }
    }

    pub fn submit(&self, read: R) {
        self.tune_active_readers(self.queue.push(read) / BUFFER_DEPTH_PER_READER + 1)
    }

    pub fn shutdown(&self) {
        self.wake_readers(MAX_READERS, true);

        for handle in &self.worker_handles {
            if let Some(handle) = handle.take_join() {
                handle.join().expect("read worker panicked")
            }
        }
    }

    fn tune_active_readers(&self, target_num: usize) {
        let observed_num = self.active_readers.load(Ordering::Relaxed);
        self.active_readers.store(target_num, Ordering::Relaxed);
        if target_num > observed_num {
            self.wake_readers(target_num, false);
        }
    }

    fn wake_readers(&self, n: usize, force: bool) {
        for worker in self.worker_handles.iter().take(n) {
            if force || worker.is_parked() {
                worker.wake();
            }
        }
    }
}
