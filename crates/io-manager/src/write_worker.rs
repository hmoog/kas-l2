use std::{
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    thread::JoinHandle,
};

use crossbeam_utils::{
    CachePadded,
    sync::{Parker, Unparker},
};
use kas_l2_io_core::KVStore;

use crate::{WriteCmd, cmd_queue::CmdQueue, config::BATCH_SIZE};

pub struct WriteWorker<K: KVStore, W: WriteCmd<K::Namespace>> {
    store: Arc<K>,
    queue: CmdQueue<W>,
    parked: Arc<CachePadded<AtomicBool>>,
    parker: Parker,
    is_shutdown: Arc<AtomicBool>,
}

impl<K: KVStore, W: WriteCmd<K::Namespace>> WriteWorker<K, W> {
    pub fn new(
        queue: CmdQueue<W>,
        parked: Arc<CachePadded<AtomicBool>>,
        store: Arc<K>,
        is_shutdown: Arc<AtomicBool>,
    ) -> Self {
        Self {
            queue,
            parked,
            store,
            parker: Parker::new(),
            is_shutdown,
        }
    }

    pub fn unparker(&self) -> Unparker {
        self.parker.unparker().clone()
    }

    pub fn start(self) -> JoinHandle<()> {
        thread::spawn(move || self.run())
    }

    fn run(self) {
        let mut batch = self.store.new_batch();
        let mut batch_size = 0;

        while !self.is_shutdown.load(Ordering::Acquire) {
            match self.queue.pop() {
                (Some(cmd), _) => {
                    cmd.exec(&batch);
                    batch_size += 1;

                    if batch_size >= BATCH_SIZE {
                        self.store
                            .write_batch(batch)
                            .expect("failed to write batch");
                        batch = self.store.new_batch();
                        batch_size = 0;
                    }
                }
                _ => self.park(),
            }
        }

        if batch_size > 0 {
            self.store
                .write_batch(batch)
                .expect("failed to write batch");
        }
    }

    fn park(&self) {
        self.parked.store(true, Ordering::Relaxed);
        while let (Some(cmd), _) = self.queue.pop() {
            cmd.exec(self.store.deref());
        }
        self.parker.park();
    }
}
