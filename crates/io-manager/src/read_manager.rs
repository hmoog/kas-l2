use std::{
    array,
    marker::PhantomData,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread::JoinHandle,
};

use crossbeam_queue::ArrayQueue;
use crossbeam_utils::{CachePadded, atomic::AtomicCell, sync::Unparker};
use kas_l2_io_core::KVStore;

use crate::{
    ReadCmd,
    cmd_queue::CmdQueue,
    config::{BUFFER_DEPTH_PER_READER, MAX_READERS},
    read_worker::ReadWorker,
};

pub struct ReadManager<K: KVStore, R: ReadCmd<K::Namespace>> {
    queue: CmdQueue<R>,
    workers_active: Arc<CachePadded<AtomicUsize>>,
    parked_workers: Arc<ArrayQueue<usize>>,
    unparkers: [Unparker; MAX_READERS],
    worker_handles: [AtomicCell<Option<JoinHandle<()>>>; MAX_READERS],
    _marker: PhantomData<K>,
}

impl<K: KVStore, R: ReadCmd<K::Namespace>> ReadManager<K, R> {
    pub fn new(store: Arc<K>, shutdown_flag: Arc<AtomicBool>) -> Self {
        let queue = CmdQueue::new();
        let workers_active = Arc::new(CachePadded::new(AtomicUsize::new(MAX_READERS)));
        let parked_workers = Arc::new(ArrayQueue::new(MAX_READERS));
        let workers: [ReadWorker<K, R>; MAX_READERS] = array::from_fn(|i| {
            ReadWorker::new(
                i,
                queue.clone(),
                store.clone(),
                parked_workers.clone(),
                workers_active.clone(),
                shutdown_flag.clone(),
            )
        });

        Self {
            queue,
            workers_active,
            parked_workers,
            unparkers: array::from_fn(|i| workers[i].unparker()),
            worker_handles: workers
                .map(|w| AtomicCell::new(Some(w.start(Self::park_worker_threshold)))),
            _marker: PhantomData,
        }
    }

    pub fn submit(&self, read: R) {
        let queue_len = self.queue.push(read);

        let active = self.workers_active.load(Ordering::Acquire);
        if queue_len >= Self::wake_worker_threshold(active) {
            self.wake_additional_reader(active);
        }
    }

    pub fn shutdown(&self) {
        for unparker in &self.unparkers {
            unparker.unpark()
        }
        for handle in &self.worker_handles {
            if let Some(handle) = handle.take() {
                handle.join().expect("read worker panicked")
            }
        }
    }

    fn wake_additional_reader(&self, index: usize) {
        if self
            .workers_active
            .compare_exchange(index, index + 1, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        {
            self.unparkers[self.parked_workers.pop().expect("no unparked worker")].unpark();
        }
    }

    fn wake_worker_threshold(index: usize) -> usize {
        if index >= MAX_READERS {
            usize::MAX
        } else {
            index * BUFFER_DEPTH_PER_READER
        }
    }

    fn park_worker_threshold(index: usize) -> usize {
        if index == 0 {
            0
        } else {
            index * BUFFER_DEPTH_PER_READER - BUFFER_DEPTH_PER_READER / 2
        }
    }
}
