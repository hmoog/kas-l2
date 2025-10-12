use std::{sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
}};
use std::thread::JoinHandle;
use crossbeam_utils::CachePadded;
use crossbeam_utils::sync::{Parker, Unparker};

use crate::{
    Transaction,
    io::{
        adaptive_readers::AdaptiveReaders, cmd::WriteCmd, consts::BATCH_SIZE, job_queue::JobQueue,
        kv_store::KVStore,
    },
};
use crate::io::writer_worker::WriterWorker;

pub struct BatchWriter<T: Transaction> {
    queue: JobQueue<WriteCmd<T>>,
    writer_parked: Arc<CachePadded<AtomicBool>>,
    writer_unparker: Unparker,
    writer_handle: JoinHandle<()>,
}

impl<T: Transaction> BatchWriter<T> {
    pub fn new<S: KVStore>(store: Arc<S>) -> Self {
        let queue = JobQueue::new();
        let writer_parked = Arc::new(CachePadded::new(AtomicBool::new(false)));
        let writer = WriterWorker::new(queue.clone(), writer_parked.clone(), store);

        Self {
            queue,
            writer_parked,
            writer_unparker: writer.unparker(),
            writer_handle: writer.start(),
        }
    }

    pub fn submit(&self, write: WriteCmd<T>) {
        if self.queue.push(write) >= BATCH_SIZE && self.writer_parked() {
            self.unpark_writer();
        }
    }

    #[inline(always)]
    fn writer_parked(&self) -> bool {
        self.writer_parked.load(Ordering::Relaxed)
    }

    fn unpark_writer(&self) {
        if self.writer_parked.swap(false, Ordering::Relaxed) {
            self.writer_unparker.unpark();
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

pub struct IoManager<T: Transaction, S: KVStore> {
    store: Arc<S>,
    readers: AdaptiveReaders<T, S>,
    writer: BatchWriter<T>,
}

impl<T: Transaction, S: KVStore> IoManager<T, S> {
    pub fn new(store: S) -> Self {
        let store = Arc::new(store);

        Self {
            readers: AdaptiveReaders::new(store.clone()),
            writer: BatchWriter::new(store.clone()),
            store,
        }
    }

    pub fn readers(&self) -> &AdaptiveReaders<T, S> {
        &self.readers
    }

    pub fn writer(&self) -> &BatchWriter<T> {
        &self.writer
    }
}
