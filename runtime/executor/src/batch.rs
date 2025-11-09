use crossbeam_deque::Worker;

use crate::task::Task;

pub trait Batch<T: Task>: Clone + Send + Sync + 'static {
    fn steal_available_tasks(&self, worker: &Worker<T>) -> Option<T>;

    fn is_depleted(&self) -> bool;
}
