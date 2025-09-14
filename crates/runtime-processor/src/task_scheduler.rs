use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use crossbeam_deque::{Injector, Steal, Worker as WorkerQueue};
use intrusive_collections::{LinkedList, LinkedListLink, intrusive_adapter};
use kas_l2_runtime_scheduler::{Batch, ScheduledTask, Task};

use crate::{task_scheduler::linked_list_element::*, worker_manager::WorkerManager};

pub struct TaskScheduler<T: Task> {
    id: usize,
    worker_manager: Arc<WorkerManager<T>>,
    local_queue: WorkerQueue<Arc<ScheduledTask<T>>>,
    global_queue: LinkedList<Adapter<Arc<Batch<T>>>>,
    batch_injector: Arc<Injector<Arc<Batch<T>>>>,
}

impl<T: Task> TaskScheduler<T> {
    pub fn new(
        id: usize,
        worker_manager: Arc<WorkerManager<T>>,
        local_queue: WorkerQueue<Arc<ScheduledTask<T>>>,
        batch_injector: Arc<Injector<Arc<Batch<T>>>>,
    ) -> Self {
        Self {
            id,
            worker_manager,
            local_queue,
            global_queue: LinkedList::new(Adapter::new()),
            batch_injector,
        }
    }

    pub fn pop_task(&mut self) -> Option<Arc<ScheduledTask<T>>> {
        loop {
            if let Some(task) = self.local_queue.pop() {
                return Some(task);
            }

            if let Some(task) = self.steal_from_global_queue() {
                return Some(task);
            }

            if let Some(task) = self.steal_from_other_workers() {
                return Some(task);
            }

            if self.try_update_global_queue() {
                continue;
            }

            break None;
        }
    }

    pub fn steal_from_other_workers(&mut self) -> Option<Arc<ScheduledTask<T>>> {
        for (j, stealer) in self.worker_manager.stealers.iter().enumerate() {
            if j != self.id {
                loop {
                    match stealer.steal() {
                        Steal::Success(task) => return Some(task),
                        Steal::Retry => continue,
                        Steal::Empty => break,
                    }
                }
            }
        }
        None
    }

    pub fn steal_from_global_queue(&mut self) -> Option<Arc<ScheduledTask<T>>> {
        let mut curr_element = self.global_queue.cursor_mut();
        while let Some(batch) = curr_element.get() {
            let pending_tasks = batch.pending_tasks();
            if pending_tasks.is_done() {
                curr_element.remove();
                continue;
            }

            loop {
                match pending_tasks.ready.steal_batch_and_pop(&self.local_queue) {
                    Steal::Success(task) => return Some(task),
                    Steal::Retry => continue,
                    Steal::Empty => break,
                }
            }
            curr_element.move_next();
        }
        None
    }

    pub fn try_update_global_queue(&mut self) -> bool {
        let mut updated = false;
        loop {
            match self.batch_injector.steal() {
                Steal::Success(batch) => {
                    self.global_queue
                        .push_back(Box::new(LinkedListElement::new(batch)));
                    updated = true;
                }
                Steal::Empty => break,
                Steal::Retry => (),
            }
        }
        updated
    }
}

mod linked_list_element {
    use super::*;

    pub struct LinkedListElement<T> {
        pub(super) link: LinkedListLink,
        pub inner: T,
    }

    impl<T> LinkedListElement<T> {
        pub fn new(inner: T) -> Self {
            Self {
                link: LinkedListLink::new(),
                inner,
            }
        }
    }

    impl<T> Deref for LinkedListElement<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    impl<T> DerefMut for LinkedListElement<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }

    intrusive_adapter!(pub Adapter<T> = Box<LinkedListElement<T>>: LinkedListElement<T> { link: LinkedListLink });
}
