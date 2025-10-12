use std::{
    array,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread::JoinHandle,
};

use crossbeam_queue::ArrayQueue;
use crossbeam_utils::{CachePadded, sync::Unparker};

use crate::{
    Transaction,
    io::{
        cmd::ReadCommands,
        consts::{BUFFER_DEPTH_PER_READER, MAX_READERS},
        job_queue::JobQueue,
        kv_store::KVStore,
        read_worker::ReadWorker,
    },
};

pub struct AdaptiveReaders<T: Transaction, S: KVStore> {
    store: Arc<S>,
    queue: JobQueue<ReadCommands<T>>,
    workers_active: Arc<CachePadded<AtomicUsize>>,
    parked_workers: Arc<ArrayQueue<usize>>,
    unparkers: [Unparker; MAX_READERS],
    worker_handles: [JoinHandle<()>; MAX_READERS],
}

impl<T: Transaction, S: KVStore> AdaptiveReaders<T, S> {
    pub fn new(store: Arc<S>) -> Self {
        let queue = JobQueue::new();
        let workers_active = Arc::new(CachePadded::new(AtomicUsize::new(MAX_READERS)));
        let parked_workers = Arc::new(ArrayQueue::new(MAX_READERS));
        let workers: [ReadWorker<S, T>; MAX_READERS] = array::from_fn(|i| {
            ReadWorker::new(
                i,
                queue.clone(),
                store.clone(),
                parked_workers.clone(),
                workers_active.clone(),
            )
        });

        Self {
            store,
            queue,
            workers_active,
            parked_workers,
            unparkers: array::from_fn(|i| workers[i].unparker()),
            worker_handles: workers.map(|w| w.start(Self::park_worker_threshold)),
        }
    }

    pub fn submit(&self, read: ReadCommands<T>) {
        let queue_len = self.queue.push(read);

        let active = self.workers_active.load(Ordering::Acquire);
        if queue_len >= Self::wake_worker_threshold(active) {
            self.wake_additional_reader(active);
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
