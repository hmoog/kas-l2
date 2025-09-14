use std::{sync::Arc, thread::JoinHandle};

use kas_l2_runtime_scheduler::{Batch, Task};

use crate::workers_api::WorkersAPI;

pub struct BatchProcessor<T: Task> {
    workers_api: Arc<WorkersAPI<T>>,
    worker_handles: Vec<JoinHandle<()>>,
}

impl<T: Task> BatchProcessor<T> {
    pub fn new(worker_count: usize) -> Self {
        let (workers_api, worker_handles) = WorkersAPI::new_with_workers(worker_count);
        Self {
            workers_api,
            worker_handles,
        }
    }

    pub fn inject(&self, batch: Arc<Batch<T>>) {
        self.workers_api.inject_batch(batch);
    }

    pub fn shutdown(self) {
        self.workers_api.shutdown.open();
        for handle in self.worker_handles {
            let _ = handle.join();
        }
    }
}
