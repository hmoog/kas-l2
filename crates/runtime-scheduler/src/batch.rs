use crate::task::Task;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub struct Batch<T: Task> {
    tasks: Vec<T>,
    done: Vec<AtomicBool>,
    pending_tasks: AtomicU64,
}

impl<T: Task> Batch<T> {
    pub fn new(tasks: Vec<T>) -> Self {
        let done = (0..tasks.len()).map(|_| AtomicBool::new(false)).collect();
        let pending_tasks = AtomicU64::new(tasks.len() as u64);

        Self {
            done,
            tasks,
            pending_tasks,
        }
    }

    pub fn mark_done(&self, index: usize) -> bool {
        let Some(flag) = self.done.get(index) else {
            return false;
        };

        flag.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
            && self.pending_tasks.fetch_sub(1, Ordering::SeqCst) == 1
    }

    pub fn tasks(&self) -> &[T] {
        &self.tasks
    }

    pub fn is_complete(&self) -> bool {
        self.pending_tasks.load(Ordering::SeqCst) == 0
    }

    pub fn remaining(&self) -> u64 {
        self.pending_tasks.load(Ordering::SeqCst)
    }
}
