pub struct WorkerPool {
    worker_count: usize,
}

impl WorkerPool {
    pub fn new(worker_count: usize) -> Self {
        Self { worker_count }
    }

    pub fn worker_count(&self) -> usize {
        self.worker_count
    }
}