use std::{
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    thread::JoinHandle,
};

use crossbeam_queue::ArrayQueue;
use crossbeam_utils::{
    CachePadded,
    sync::{Parker, Unparker},
};

use crate::{
    Transaction,
    io::{cmd::ReadCommands, job_queue::JobQueue, kv_store::KVStore},
};

pub struct ReadWorker<S: KVStore, T: Transaction> {
    id: usize,
    queue: JobQueue<ReadCommands<T>>,
    store: Arc<S>,
    parked_workers: Arc<ArrayQueue<usize>>,
    readers_active: Arc<CachePadded<AtomicUsize>>,
    parker: Parker,
}

impl<S: KVStore, T: Transaction> ReadWorker<S, T> {
    pub fn new(
        id: usize,
        queue: JobQueue<ReadCommands<T>>,
        store: Arc<S>,
        parked_workers: Arc<ArrayQueue<usize>>,
        readers_active: Arc<CachePadded<AtomicUsize>>,
    ) -> Self {
        Self {
            id,
            queue,
            store,
            parked_workers,
            readers_active,
            parker: Parker::new(),
        }
    }

    pub fn unparker(&self) -> Unparker {
        self.parker.unparker().clone()
    }

    pub fn start<F>(self, park_threshold: F) -> JoinHandle<()>
    where
        F: Fn(usize) -> usize + Send + Sync + 'static,
    {
        thread::spawn(move || self.run(park_threshold))
    }

    fn run<F: Fn(usize) -> usize>(self, adaptive_park_threshold: F) {
        loop {
            match self.queue.pop() {
                (Some(cmd), approx_queue_len) => {
                    cmd.exec(self.store.deref());

                    if approx_queue_len < adaptive_park_threshold(self.id) {
                        self.park()
                    }
                }
                _ => self.park(),
            }
        }
    }

    fn park(&self) {
        self.parked_workers
            .push(self.id)
            .expect("parked_workers full");
        if self.readers_active.fetch_sub(1, Ordering::Release) == 1 {
            while let (Some(cmd), _) = self.queue.pop() {
                cmd.exec(self.store.deref());
            }
        }
        self.parker.park();
    }
}
