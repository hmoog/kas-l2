use std::{
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    thread::JoinHandle,
};
use std::sync::atomic::AtomicBool;
use crossbeam_queue::ArrayQueue;
use crossbeam_utils::{
    CachePadded,
    sync::{Parker, Unparker},
};

use crate::{
    Transaction,
    io::{cmd::WriteCmd, job_queue::JobQueue, kv_store::KVStore},
};

pub struct WriterWorker<S: KVStore, T: Transaction> {
    queue: JobQueue<WriteCmd<T>>,
    writer_parked: Arc<CachePadded<AtomicBool>>,
    store: Arc<S>,
    parker: Parker,
}

impl<S: KVStore, T: Transaction> WriterWorker<S, T> {
    pub fn new(
        queue: JobQueue<WriteCmd<T>>,
        writer_parked: Arc<CachePadded<AtomicBool>>,
        store: Arc<S>,
    ) -> Self {
        Self {
            queue,
            writer_parked,
            store,
            parker: Parker::new(),
        }
    }

    pub fn unparker(&self) -> Unparker {
        self.parker.unparker().clone()
    }

    pub fn start(self) -> JoinHandle<()> {
        thread::spawn(move || self.run())
    }

    fn run(self) {
        loop {
            match self.queue.pop() {
                (Some(cmd), _) => {
                    cmd.exec(self.store.deref());
                }
                _ => self.park(),
            }
        }
    }

    fn park(&self) {
        self.writer_parked.store(true, Ordering::Relaxed);
        while let (Some(cmd), _) = self.queue.pop() {
            cmd.exec(self.store.deref());
        }
        self.parker.park();
    }
}
