use std::{marker::PhantomData, sync::Arc};

use crossbeam_deque::Worker;
use crossbeam_queue::ArrayQueue;
use intrusive_collections::LinkedList;
use tracing::trace;

use crate::{Batch, batch_queue::linked_list::Adapter, task::Task};

pub struct BatchQueue<T: Task, B: Batch<T>> {
    queue: LinkedList<Adapter<B>>,
    new_batches: Arc<ArrayQueue<B>>,
    _marker: PhantomData<T>,
}

impl<T: Task, B: Batch<T>> BatchQueue<T, B> {
    pub fn new(new_batches: Arc<ArrayQueue<B>>) -> Self {
        Self { queue: LinkedList::new(Adapter::new()), new_batches, _marker: PhantomData }
    }

    pub fn steal(&mut self, worker_queue: &Worker<T>) -> Option<T> {
        loop {
            let mut queue_element = self.queue.cursor_mut();
            queue_element.move_next();

            while let Some(batch) = queue_element.get() {
                if let Some(transaction) = batch.steal_available_tasks(worker_queue) {
                    trace!("stole task from batch");
                    return Some(transaction);
                }

                if batch.is_depleted() {
                    trace!("batch depleted, removing");
                    queue_element.remove();
                } else {
                    queue_element.move_next();
                }
            }

            if !self.try_pull_new_batches() {
                trace!("no new batches to pull");
                return None;
            }
        }
    }

    fn try_pull_new_batches(&mut self) -> bool {
        let mut pulled = false;
        while let Some(batch) = self.new_batches.pop() {
            trace!("pulled new batch from inbox");
            self.queue.push_back(Box::new(linked_list::Element::new(batch)));
            pulled = true;
        }
        pulled
    }
}

mod linked_list {
    use std::ops::{Deref, DerefMut};

    use intrusive_collections::{LinkedListLink, intrusive_adapter};

    pub struct Element<T> {
        pub link: LinkedListLink,
        pub inner: T,
    }

    impl<T> Element<T> {
        pub fn new(inner: T) -> Self {
            Self { link: LinkedListLink::new(), inner }
        }
    }

    impl<T> Deref for Element<T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    impl<T> DerefMut for Element<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }

    intrusive_adapter!(pub Adapter<T> = Box<Element<T>>: Element<T> { link: LinkedListLink });
}
