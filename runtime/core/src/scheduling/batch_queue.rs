use std::sync::Arc;

use crossbeam_deque::Worker;
use crossbeam_queue::ArrayQueue;
use intrusive_collections::LinkedList;
use kas_l2_storage_manager::Store;

use crate::{Batch, RuntimeState, RuntimeTx, Vm};

pub struct BatchQueue<S: Store<StateSpace = RuntimeState>, VM: Vm> {
    queue: LinkedList<linked_list::Adapter<Batch<S, VM>>>,
    new_batches: Arc<ArrayQueue<Batch<S, VM>>>,
}

impl<S: Store<StateSpace = RuntimeState>, VM: Vm> BatchQueue<S, VM> {
    pub fn new(new_batches: Arc<ArrayQueue<Batch<S, VM>>>) -> Self {
        Self { queue: LinkedList::new(linked_list::Adapter::new()), new_batches }
    }

    pub fn steal(&mut self, worker_queue: &Worker<RuntimeTx<S, VM>>) -> Option<RuntimeTx<S, VM>> {
        loop {
            let mut queue_element = self.queue.cursor_mut();
            queue_element.move_next();

            while let Some(batch) = queue_element.get() {
                if let Some(transaction) = batch.steal_available_txs(worker_queue) {
                    return Some(transaction);
                }

                if batch.is_depleted() {
                    queue_element.remove();
                } else {
                    queue_element.move_next();
                }
            }

            if !self.try_pull_new_batches() {
                return None;
            }
        }
    }

    fn try_pull_new_batches(&mut self) -> bool {
        let mut pulled = false;
        while let Some(batch) = self.new_batches.pop() {
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
