use std::sync::Arc;

use crossbeam_deque::{Injector, Steal, Worker as WorkerQueue};
use intrusive_collections::LinkedList;
use kas_l2_causal_scheduler::{Batch, ScheduledTask, Task};

use crate::batch_injector::linked_list_element::*;

pub struct BatchInjector<T: Task> {
    queue: LinkedList<Adapter<Arc<Batch<T>>>>,
    injector: Arc<Injector<Arc<Batch<T>>>>,
}

impl<T: Task> BatchInjector<T> {
    pub fn new(injector: Arc<Injector<Arc<Batch<T>>>>) -> Self {
        Self {
            queue: LinkedList::new(Adapter::new()),
            injector,
        }
    }

    pub fn steal(
        &mut self,
        local_queue: &WorkerQueue<Arc<ScheduledTask<T>>>,
    ) -> Option<Arc<ScheduledTask<T>>> {
        loop {
            let mut curr_element = self.queue.cursor_mut();
            curr_element.move_next();
            while let Some(batch) = curr_element.get() {
                let pending_tasks = batch.pending_tasks();
                if pending_tasks.is_done() {
                    curr_element.remove();
                    continue;
                }

                loop {
                    match pending_tasks.ready.steal_batch_and_pop(local_queue) {
                        Steal::Success(task) => return Some(task),
                        Steal::Retry => continue,
                        Steal::Empty => break,
                    }
                }
                curr_element.move_next();
            }

            if !self.update() {
                break None;
            }
        }
    }

    fn update(&mut self) -> bool {
        let mut updated = false;
        loop {
            match self.injector.steal() {
                Steal::Success(batch) => {
                    self.queue
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
    use std::ops::{Deref, DerefMut};

    use intrusive_collections::{LinkedListLink, intrusive_adapter};

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
