use std::{sync::Arc, thread::JoinHandle};
use crate::task::Task;
use crate::worker_manager::WorkerManager;

pub struct Processor<T: Task> {
    manager: Arc<WorkerManager<T>>,
    handles: Vec<JoinHandle<()>>,
}

impl<T: Task> Processor<T> {
    pub fn new(worker_count: usize) -> Self {
        let (manager, handles) = WorkerManager::spawn(worker_count);
        Self { manager, handles }
    }

    pub fn resume(&self) {
        for unparker in &self.manager.unparkers {
            unparker.unpark();
        }
    }
}
