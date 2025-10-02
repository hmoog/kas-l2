use std::sync::Arc;

use crossbeam_deque::{Injector, Steal, Worker as WorkerQueue};
use intrusive_collections::LinkedList;

use crate::{BatchApi, RuntimeTx, Transaction, execution::batch_injector::linked_list::*};

pub struct BatchInjector<T: Transaction> {
    queue: LinkedList<Adapter<BatchApi<T>>>,
    injector: Arc<Injector<BatchApi<T>>>,
}

impl<T: Transaction> BatchInjector<T> {
    pub fn new(injector: Arc<Injector<BatchApi<T>>>) -> Self {
        Self {
            queue: LinkedList::new(Adapter::new()),
            injector,
        }
    }

    pub fn steal(&mut self, worker_queue: &WorkerQueue<RuntimeTx<T>>) -> Option<RuntimeTx<T>> {
        loop {
            let mut queue_element = self.queue.cursor_mut();
            queue_element.move_next();

            while let Some(batch) = queue_element.get() {
                if let Some(transaction) = batch.steal_available_txs(worker_queue) {
                    return Some(transaction);
                }

                if batch.pending_txs() == 0 && batch.available_txs() == 0 {
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
        let mut success = false;
        loop {
            match self.injector.steal() {
                Steal::Success(batch) => {
                    self.queue.push_back(Box::new(Element::new(batch)));
                    success = true;
                }
                Steal::Empty => break,
                Steal::Retry => continue,
            }
        }
        success
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
            Self {
                link: LinkedListLink::new(),
                inner,
            }
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
