use std::{
    sync::{Arc, atomic::Ordering},
    thread,
    thread::JoinHandle,
};

use crossbeam_deque::{Steal, Worker as WorkerQueue};
use crossbeam_utils::sync::Parker;
use crate::task::Task;
use crate::worker_manager::WorkerManager;

pub struct Worker<T: Task> {
    pub(crate) id: usize,
    pub(crate) queue: WorkerQueue<T>,
    pub(crate) parker: Parker,
}

impl<T: Task> Worker<T> {
    pub(crate) fn new(id: usize) -> Self {
        Self {
            id,
            queue: WorkerQueue::new_fifo(),
            parker: Parker::new(),
        }
    }

    pub(crate) fn start(self, settings: Arc<WorkerManager<T>>) -> JoinHandle<()> {
        thread::spawn(move || self.run(settings))
    }

    pub(crate) fn run(self, settings: Arc<WorkerManager<T>>) {
        let mut injectors_version = settings.injectors_version.load(Ordering::Relaxed);
        let mut injectors = settings.injectors.load();

        while !settings.shutdown.is_open() {
            // 1. Run local
            if let Some(_task) = self.queue.pop() {
                // TODO: RUN TASK
                continue;
            }

            // 2. Scan injectors oldest → newest
            let mut found = false;
            for injector in injectors.iter() {
                match injector.steal_batch_and_pop(&self.queue) {
                    Steal::Success(_task) => {
                        // TODO: RUN TASK
                        found = true;
                        break;
                    }
                    Steal::Retry => continue,
                    Steal::Empty => {}
                }
            }
            if found {
                continue;
            }

            // 3. Steal from other workers
            for (j, stealer) in settings.stealers.iter().enumerate() {
                if j == self.id {
                    continue;
                }
                match stealer.steal() {
                    Steal::Success(_task) => {
                        // TODO: RUN TASK
                        found = true;
                        break;
                    }
                    Steal::Retry => continue,
                    Steal::Empty => {}
                }
            }
            if found {
                continue;
            }

            // 4. check if there are new injectors
            if injectors_version != settings.injectors_version.load(Ordering::Acquire) {
                injectors_version = settings.injectors_version.load(Ordering::Relaxed);
                injectors = settings.injectors.load();
                continue;
            }

            self.parker.park();
        }
    }
}
