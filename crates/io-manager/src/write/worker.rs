use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use crossbeam_utils::{CachePadded, sync::Parker};
use kas_l2_io_core::KVStore;

use crate::{WriteCmd, cmd_queue::CmdQueue, config::BATCH_SIZE, worker_handle::WorkerHandle};

pub struct WriteWorker<K: KVStore, W: WriteCmd<K::Namespace>> {
    store: Arc<K>,
    queue: CmdQueue<W>,
    is_parked: Arc<CachePadded<AtomicBool>>,
    parker: Parker,
    is_shutdown: Arc<AtomicBool>,
}

impl<K: KVStore, W: WriteCmd<K::Namespace>> WriteWorker<K, W> {
    pub(crate) fn spawn(
        queue: &CmdQueue<W>,
        store: &Arc<K>,
        is_shutdown: &Arc<AtomicBool>,
    ) -> WorkerHandle {
        let this = Self {
            queue: queue.clone(),
            store: store.clone(),
            is_shutdown: is_shutdown.clone(),
            parker: Parker::new(),
            is_parked: Arc::new(CachePadded::new(AtomicBool::new(false))),
        };

        WorkerHandle::new(
            this.parker.unparker().clone(),
            this.is_parked.clone(),
            thread::spawn(move || this.run()),
        )
    }

    fn run(self) {
        let mut batch = self.store.new_batch();
        let mut batch_size = 0;

        while !self.is_shutdown() {
            match self.queue.pop() {
                (Some(cmd), _) => {
                    cmd.exec(&batch);
                    batch_size += 1;

                    if batch_size >= BATCH_SIZE {
                        self.store.write_batch(batch).expect("write batch failed");
                        batch = self.store.new_batch();
                        batch_size = 0;
                    }
                }
                _ => self.park(),
            }
        }

        if batch_size > 0 {
            self.store.write_batch(batch).expect("write batch failed");
        }
    }

    #[inline(always)]
    fn is_shutdown(&self) -> bool {
        self.is_shutdown.load(Ordering::Acquire)
    }

    fn park(&self) {
        self.is_parked.store(true, Ordering::Relaxed);
        if !self.is_shutdown() && self.queue.is_empty() {
            self.parker.park_timeout(Duration::from_millis(100));
        }
        self.is_parked.store(false, Ordering::Relaxed);
    }
}
