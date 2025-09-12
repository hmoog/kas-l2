use std::thread::JoinHandle;
use crossbeam_utils::sync::Parker;

pub struct Worker<T> {
    id: usize,
    join_handle: JoinHandle<()>,
    local_queue: crossbeam_deque::Worker<T>,
}

impl<T> Worker<T> {
    pub fn new(id: usize, join_handle: JoinHandle<()>) -> Self {
        let worker = crossbeam_deque::Worker::new_fifo();
        let stealer = worker.stealer();
        let parker = Parker::new();
        let unparker = parker.unparker().clone();

        Self { id, join_handle, local_queue: worker }
    }

    pub fn stealer(&self) -> crossbeam_deque::Stealer<T> {
        self.local_queue.stealer()
    }

    pub fn id(&self) -> usize {
        self.id
    }
}