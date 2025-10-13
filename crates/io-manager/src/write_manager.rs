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

    // pub fn start_io(&self) {
    //     const MAX_WAIT: Duration = Duration::from_millis(10);
    //
    //     let parker = Parker::new();
    //     let unparker = parker.unparker().clone();
    //
    //     loop {
    //         let mut buffer = Vec::with_capacity(MAX_BATCH_SIZE);
    //         let start = Instant::now();
    //
    //         // 1. Accumulate up to N jobs or until timeout/commit
    //         while buffer.len() < MAX_BATCH_SIZE && start.elapsed() < MAX_WAIT {
    //             if let Some(job) = self.writer_queue.pop() {
    //                 buffer.push(job);
    //                 if matches!(buffer.last(), Some(WriteCmd::CommitBatch { .. })) {
    //                     break; // force flush on commit
    //                 }
    //             } else {
    //                 parker.park_timeout();
    //                 break;
    //             }
    //         }
    //
    //         if buffer.is_empty() {
    //             continue;
    //         }
    //
    //         // 2. Build single RocksDB WriteBatch
    //         let mut wb_ops = Vec::new();
    //         let mut callbacks = Vec::new();
    //
    //         for job in buffer {
    //             match job {
    //                 IoJob::Put {
    //                     ns,
    //                     key,
    //                     value,
    //                     on_complete,
    //                     ..
    //                 } => {
    //                     wb_ops.push((ns, key, value));
    //                     if let Some(cb) = on_complete {
    //                         callbacks.push(cb);
    //                     }
    //                 }
    //                 IoJob::Commit {
    //                     pointer_updates,
    //                     on_complete,
    //                     ..
    //                 } => {
    //                     wb_ops.extend(pointer_updates);
    //                     if let Some(cb) = on_complete {
    //                         callbacks.push(cb);
    //                     }
    //                     // Commit job forces flush boundary
    //                     break;
    //                 }
    //                 IoJob::Shutdown => return,
    //             }
    //         }
    //
    //         // 3. Perform one atomic RocksDB write
    //         store.write_batch(&wb_ops);
    //
    //         // 4. Only now run callbacks / notify completion
    //         for cb in callbacks {
    //             cb();
    //         }
    //
    //         // Optionally unpark producers or signal next stage
    //         unparker.unpark();
    //     }
    // }
}
