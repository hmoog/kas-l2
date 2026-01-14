use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use kas_l2_core_macros::smart_pointer;

#[smart_pointer]
pub struct RuntimeContext {
    parent_context: Option<RuntimeContextRef>,
    last_batch_index: AtomicU64,
    rollback_threshold: AtomicU64,
}

impl RuntimeContext {
    pub fn new(index: u64) -> Self {
        Self(Arc::new(RuntimeContextData {
            parent_context: None,
            last_batch_index: AtomicU64::new(index),
            rollback_threshold: AtomicU64::new(u64::MAX),
        }))
    }

    pub fn last_batch_index(&self) -> u64 {
        self.last_batch_index.load(Ordering::Acquire)
    }

    pub fn next_batch_index(&self) -> u64 {
        self.last_batch_index.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub fn rollback(&mut self, threshold: u64) {
        let parent_context = self.clone();
        Arc::make_mut(&mut self.0).parent_context = Some(parent_context.downgrade());
        parent_context.rollback_threshold.store(threshold, Ordering::Release);
    }

    pub fn rollback_threshold(&self) -> u64 {
        self.rollback_threshold.load(Ordering::Acquire)
    }
}

impl Clone for RuntimeContextData {
    fn clone(&self) -> Self {
        Self {
            parent_context: self.parent_context.clone(),
            last_batch_index: AtomicU64::new(self.last_batch_index.load(Ordering::Acquire)),
            rollback_threshold: AtomicU64::new(self.rollback_threshold.load(Ordering::Acquire)),
        }
    }
}