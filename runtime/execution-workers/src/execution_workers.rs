use std::thread::JoinHandle;

use crate::{Batch, WorkersApi, task::Task};

pub struct ExecutionWorkers<T: Task, B: Batch<T>> {
    workers: WorkersApi<T, B>,
    handles: Vec<JoinHandle<()>>,
}

impl<T: Task, B: Batch<T>> ExecutionWorkers<T, B> {
    pub fn new(worker_count: usize) -> Self {
        let (workers, handles) = WorkersApi::new_with_workers(worker_count);
        Self { workers, handles }
    }

    pub fn execute(&self, batch: B) {
        self.workers.push_batch(batch);
    }

    pub fn waker(&self) -> impl Fn() + Clone + Send + Sync + 'static {
        let workers = self.workers.clone();
        move || workers.wake_all()
    }

    pub fn shutdown(self) {
        self.workers.shutdown();
        for handle in self.handles {
            handle.join().expect("executor worker panicked");
        }
    }
}
