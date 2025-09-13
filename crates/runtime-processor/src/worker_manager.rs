use std::{
    sync::{Arc, atomic::AtomicU64},
    thread::JoinHandle,
};

use crossbeam_deque::{Injector, Stealer};
use crossbeam_utils::sync::Unparker;
use kas_l2_atomic::{AtomicArc, AtomicAsyncLatch};
use crate::task::Task;
use crate::worker::Worker;

pub struct WorkerManager<T: Task> {
    pub(crate) injectors: AtomicArc<Vec<Injector<T>>>,
    pub(crate) injectors_version: AtomicU64,
    pub(crate) stealers: Vec<Stealer<T>>,
    pub(crate) unparkers: Vec<Unparker>,
    pub(crate) shutdown: AtomicAsyncLatch,
}

impl<T: Task> WorkerManager<T> {
    pub fn spawn(worker_count: usize) -> (Arc<Self>, Vec<JoinHandle<()>>) {
        let mut this = Self {
            injectors: AtomicArc::new(Arc::new(vec![])),
            injectors_version: AtomicU64::new(0),
            stealers: vec![],
            unparkers: vec![],
            shutdown: AtomicAsyncLatch::new(),
        };

        let workers = this.create_workers(worker_count);
        this.start_workers(workers)
    }

    // pub fn register_injector(&self, injector: Injector<T>) {
    //     // Load the current list
    //     let mut injectors = (*self.injectors.load()).clone();
    //
    //     // Append new one
    //     injectors.push(injector);
    //
    //     // Replace atomically
    //     self.injectors.store(Arc::new(injectors));
    //
    //     // Bump version so workers refresh
    //     self.injectors_version.fetch_add(1, std::sync::atomic::Ordering::Release);
    //
    //     // Wake sleeping workers
    //     for u in &self.unparkers {
    //         u.unpark();
    //     }
    // }

    fn create_workers(&mut self, worker_count: usize) -> Vec<Worker<T>> {
        (0..worker_count).map(|id| self.create_worker(id)).collect()
    }

    fn start_workers(self, workers: Vec<Worker<T>>) -> (Arc<Self>, Vec<JoinHandle<()>>) {
        let this = Arc::new(self);
        let handles = workers.into_iter().map(|w| w.start(this.clone())).collect();
        (this, handles)
    }

    fn create_worker(&mut self, id: usize) -> Worker<T> {
        let worker = Worker::new(id);
        self.stealers.push(worker.queue.stealer());
        self.unparkers.push(worker.parker.unparker().clone());
        worker
    }
}
