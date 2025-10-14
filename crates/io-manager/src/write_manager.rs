use std::{
    marker::PhantomData,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
};

use crossbeam_utils::{CachePadded, atomic::AtomicCell, sync::Unparker};
use kas_l2_io_core::KVStore;

use crate::{WriteCmd, cmd_queue::CmdQueue, config::BATCH_SIZE, write_worker::WriteWorker};

pub struct WriteManager<K: KVStore, W: WriteCmd<K::Namespace>> {
    queue: CmdQueue<W>,
    parked: Arc<CachePadded<AtomicBool>>,
    unparker: Unparker,
    handle: AtomicCell<Option<JoinHandle<()>>>,
    _marker: PhantomData<K>,
}

impl<K: KVStore, W: WriteCmd<K::Namespace>> WriteManager<K, W> {
    pub fn new(store: Arc<K>, is_shutdown: Arc<AtomicBool>) -> Self {
        let queue = CmdQueue::new();
        let writer_parked = Arc::new(CachePadded::new(AtomicBool::new(false)));
        let writer = WriteWorker::new(queue.clone(), writer_parked.clone(), store, is_shutdown);

        Self {
            queue,
            parked: writer_parked,
            unparker: writer.unparker(),
            handle: AtomicCell::new(Some(writer.start())),
            _marker: PhantomData,
        }
    }

    pub fn submit(&self, write: W) {
        if self.queue.push(write) >= BATCH_SIZE && self.writer_parked() {
            self.unpark_writer();
        }
    }

    pub fn shutdown(&self) {
        self.unparker.unpark();

        if let Some(handle) = self.handle.take() {
            handle.join().expect("write worker panicked");
        }
    }

    #[inline(always)]
    fn writer_parked(&self) -> bool {
        self.parked.load(Ordering::Relaxed)
    }

    fn unpark_writer(&self) {
        if self.parked.swap(false, Ordering::Relaxed) {
            self.unparker.unpark();
        }
    }
}
