use std::{sync::Arc, thread, thread::JoinHandle};

use crossbeam_deque::{Injector, Worker as WorkerQueue};
use crossbeam_utils::sync::Parker;
use kas_l2_runtime_scheduler::{Batch, ScheduledTask, Task};

use crate::{global_queue::GlobalQueue, workers_api::WorkersAPI};

pub struct Worker<T: Task> {
    pub(crate) id: usize,
    pub(crate) local_queue: WorkerQueue<Arc<ScheduledTask<T>>>,
    pub(crate) parker: Parker,
    pub(crate) injector: Arc<Injector<Arc<Batch<T>>>>,
}

impl<T: Task> Worker<T> {
    pub(crate) fn new(id: usize) -> Self {
        Self {
            id,
            local_queue: WorkerQueue::new_fifo(),
            parker: Parker::new(),
            injector: Arc::new(Injector::new()),
        }
    }

    pub(crate) fn start(self, workers_api: Arc<WorkersAPI<T>>) -> JoinHandle<()> {
        thread::spawn(move || self.run(workers_api))
    }

    fn run(self, workers_api: Arc<WorkersAPI<T>>) {
        let mut global_queue = GlobalQueue::new(self.injector);

        while !workers_api.shutdown.is_open() {
            match self
                .local_queue
                .pop()
                .or_else(|| global_queue.steal(&self.local_queue))
                .or_else(|| workers_api.steal(self.id))
            {
                Some(task) => Self::process_task(task),
                None => self.parker.park(),
            }
        }
    }

    fn process_task(task: Arc<ScheduledTask<T>>) {
        // TODO: Execute Task before marking done
        task.done()
    }
}
