use std::sync::Arc;

use crossbeam_deque::{Injector, Steal, Worker as WorkerQueue};
use intrusive_collections::LinkedList;

use crate::{
    BatchApi, ScheduledTransaction, Transaction, execution::batch_injector::linked_list_element::*,
};

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

    pub fn steal(
        &mut self,
        local_queue: &WorkerQueue<ScheduledTransaction<T>>,
    ) -> Option<ScheduledTransaction<T>> {
        loop {
            let mut curr_element = self.queue.cursor_mut();
            curr_element.move_next();

            while let Some(batch) = curr_element.get() {
                if batch.pending_transactions() == 0 && batch.available_transactions() == 0 {
                    curr_element.remove();
                } else {
                    if let Some(transaction) = batch.steal_available_transactions(local_queue) {
                        return Some(transaction);
                    }

                    curr_element.move_next();
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
                    self.queue
                        .push_back(Box::new(LinkedListElement::new(batch)));
                    success = true;
                }
                Steal::Empty => break,
                Steal::Retry => continue,
            }
        }
        success
    }
}

mod linked_list_element {
    use std::ops::{Deref, DerefMut};

    use intrusive_collections::{LinkedListLink, intrusive_adapter};

    pub struct LinkedListElement<T> {
        pub link: LinkedListLink,
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
