use std::sync::Arc;

use crossbeam_deque::Worker as DequeWorker;
use crossbeam_queue::ArrayQueue;
use intrusive_collections::LinkedList;
use kas_l2_runtime_state_space::StateSpace;
use kas_l2_storage_interface::Store;

use kas_l2_runtime_execution::ExecutionBatchQueue;

use crate::{Batch, RuntimeTx, vm::VM};

pub struct BatchQueue<S: Store<StateSpace = StateSpace>, V: VM> {
    queue: LinkedList<linked_list::Adapter<Batch<S, V>>>,
    new_batches: Arc<ArrayQueue<Batch<S, V>>>,
}

impl<S: Store<StateSpace = StateSpace>, V: VM> BatchQueue<S, V> {
    pub fn new(new_batches: Arc<ArrayQueue<Batch<S, V>>>) -> Self {
        Self { queue: LinkedList::new(linked_list::Adapter::new()), new_batches }
    }

    pub fn steal(
        &mut self,
        worker_queue: &DequeWorker<RuntimeTx<S, V>>,
    ) -> Option<RuntimeTx<S, V>> {
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

impl<S, V> ExecutionBatchQueue<RuntimeTx<S, V>> for BatchQueue<S, V>
where
    S: Store<StateSpace = StateSpace>,
    V: VM,
{
    type Batch = Batch<S, V>;

    fn new(inbox: Arc<ArrayQueue<Self::Batch>>) -> Self {
        BatchQueue::new(inbox)
    }

    fn steal(&mut self, worker_queue: &DequeWorker<RuntimeTx<S, V>>) -> Option<RuntimeTx<S, V>> {
        BatchQueue::steal(self, worker_queue)
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
