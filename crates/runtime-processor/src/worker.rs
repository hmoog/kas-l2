use std::{sync::Arc, thread, thread::JoinHandle};

use crossbeam_deque::{Injector, Worker as WorkerQueue};
use crossbeam_utils::sync::Parker;
use kas_l2_runtime_scheduler::{Batch, ScheduledTask, Task};

use crate::{task_scheduler::TaskScheduler, worker_manager::WorkerManager};

/// Represents a worker thread responsible for executing scheduled tasks.
pub struct Worker<T: Task> {
    pub(crate) id: usize,
    pub(crate) queue: WorkerQueue<Arc<ScheduledTask<T>>>,
    pub(crate) parker: Parker,
    pub(crate) batch_injector: Arc<Injector<Arc<Batch<T>>>>,
}

/// Public API
impl<T: Task> Worker<T> {
    /// Creates a new worker with the specified ID.
    pub(crate) fn new(id: usize) -> Self {
        Self {
            id,
            queue: WorkerQueue::new_fifo(),
            parker: Parker::new(),
            batch_injector: Arc::new(Injector::new()),
        }
    }

    /// Starts the worker thread, returning its join handle.
    pub(crate) fn start(self, settings: Arc<WorkerManager<T>>) -> JoinHandle<()> {
        thread::spawn(move || self.run(settings))
    }
}

/// Internal methods
impl<T: Task> Worker<T> {
    /// The main execution loop for the worker, processing tasks until shutdown is signaled.
    fn run(self, worker_manager: Arc<WorkerManager<T>>) {
        let mut task_scheduler = TaskScheduler::new(
            self.id,
            worker_manager.clone(),
            self.queue,
            self.batch_injector,
        );

        while !worker_manager.shutdown.is_open() {
            match task_scheduler.pop_task() {
                Some(task) => {
                    // TODO: Execute Task before marking done
                    task.done();
                },
                None => self.parker.park(),
            }
        }
    }
}