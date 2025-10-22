use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use crossbeam_queue::SegQueue;
use crossbeam_utils::CachePadded;
use kas_l2_macros::smart_pointer;

#[smart_pointer]
pub struct CmdQueue<T> {
    queue: SegQueue<T>,
    approx_len: CachePadded<AtomicUsize>,
}

impl<T> CmdQueue<T> {
    pub fn new() -> Self {
        Self(Arc::new(CmdQueueData {
            queue: SegQueue::new(),
            approx_len: CachePadded::new(AtomicUsize::new(0)),
        }))
    }

    pub fn push(&self, job: T) -> usize {
        // bump len before pushing so that pop doesn't underflow
        let approx_new_len = self.approx_len.fetch_add(1, Ordering::Relaxed) + 1;
        self.queue.push(job);
        approx_new_len
    }

    pub fn pop(&self) -> (Option<T>, usize) {
        match self.queue.pop() {
            None => (None, self.approx_len()),
            element => (element, self.approx_len.fetch_sub(1, Ordering::Relaxed) - 1),
        }
    }

    #[inline(always)]
    pub fn approx_len(&self) -> usize {
        self.approx_len.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
