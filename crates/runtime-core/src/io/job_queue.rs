use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use crossbeam_queue::SegQueue;
use crossbeam_utils::CachePadded;
use kas_l2_runtime_macros::smart_pointer;

#[smart_pointer]
pub struct JobQueue<T> {
    queue: SegQueue<T>,
    len: CachePadded<AtomicUsize>,
}

impl<T> JobQueue<T> {
    pub fn new() -> Self {
        Self(Arc::new(JobQueueData {
            queue: SegQueue::new(),
            len: CachePadded::new(AtomicUsize::new(0)),
        }))
    }

    pub fn push(&self, job: T) -> usize {
        // bump len before pushing so that pop doesn't underflow
        let approx_new_len = self.len.fetch_add(1, Ordering::Relaxed) + 1;
        self.queue.push(job);
        approx_new_len
    }

    pub fn pop(&self) -> (Option<T>, usize) {
        match self.queue.pop() {
            None => (None, self.len.load(Ordering::Relaxed)),
            job => (job, self.len.fetch_sub(1, Ordering::Relaxed) - 1),
        }
    }

    #[inline(always)]
    pub fn approx_len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }
}
