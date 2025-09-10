use crate::batch::Batch;
use crate::task::Task;
use crossbeam_deque::Injector;
use std::sync::Arc;

pub struct Scheduler<T: Task> {
    injector: Arc<Injector<T>>,
}

impl<T: Task> Scheduler<T> {
    pub fn new() -> Self {
        Self {
            injector: Arc::new(Injector::new()),
        }
    }

    pub fn schedule(&self, batch: Batch<T>) {
        for task in batch.tasks() {
            for resource_id in task.write_locks() {}

            for resource_id in task.read_locks() {}
            task.execute();
        }
    }

    pub fn submit(&self, task: T) {
        self.injector.push(task);
    }

    pub fn injector(&self) -> &Arc<Injector<T>> {
        &self.injector
    }
}
