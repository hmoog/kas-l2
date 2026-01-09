use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use kas_l2_core_macros::smart_pointer;

#[smart_pointer]
pub struct Chain {
    last_batch_index: AtomicU64,
    rollback_threshold: AtomicU64,
}

impl Chain {
    pub fn new(index: u64) -> Self {
        Self(Arc::new(ChainData {
            last_batch_index: AtomicU64::new(index),
            rollback_threshold: AtomicU64::new(u64::MAX),
        }))
    }

    pub fn next_batch_index(&self) -> u64 {
        self.last_batch_index.fetch_add(1, Ordering::Relaxed)
    }

    pub fn rollback(&self, threshold: u64) -> Chain {
        self.rollback_threshold.store(threshold, Ordering::Release);
        Chain::new(threshold)
    }

    pub fn rollback_threshold(&self) -> u64 {
        self.rollback_threshold.load(Ordering::Acquire)
    }
}
