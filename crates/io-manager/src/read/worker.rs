use std::{
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    thread::JoinHandle,
};

use crossbeam_queue::ArrayQueue;
use crossbeam_utils::{
    CachePadded,
    sync::{Parker, Unparker},
};
use kas_l2_io_core::KVStore;

use crate::{cmd_queue::CmdQueue, read::cmd::Cmd};

pub struct ReadWorker<S: KVStore, R: Cmd<<S as KVStore>::Namespace>> {
    id: usize,
    queue: CmdQueue<R>,
    store: Arc<S>,
    parked_workers: Arc<ArrayQueue<usize>>,
    readers_active: Arc<CachePadded<AtomicUsize>>,
    parker: Parker,
    is_shutdown: Arc<AtomicBool>,
}

impl<S: KVStore, R: Cmd<<S as KVStore>::Namespace>> ReadWorker<S, R> {
    pub fn new(
        id: usize,
        queue: CmdQueue<R>,
        store: Arc<S>,
        parked_workers: Arc<ArrayQueue<usize>>,
        readers_active: Arc<CachePadded<AtomicUsize>>,
        is_shutdown: Arc<AtomicBool>,
    ) -> Self {
        Self {
            id,
            queue,
            store,
            parked_workers,
            readers_active,
            parker: Parker::new(),
            is_shutdown,
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
        while !self.is_shutdown.load(Ordering::Acquire) {
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
