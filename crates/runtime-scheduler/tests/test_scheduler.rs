use crossbeam_deque::Steal;
use kas_l2_runtime_scheduler::{Scheduler, Task};

#[test]
pub fn test_scheduler() {
    let batch = Scheduler::schedule(vec![Transaction {
        id: 0,
        write_locks: vec![1, 2],
        read_locks: vec![3, 4],
    }]);

    let injector = batch.injector();

    // Steal an element
    match injector.steal() {
        Steal::Success(task) => {
            println!("Got task with id {}", task.element().id);
            task.done()
        }
        Steal::Empty => {
            panic!("Injector was empty!");
        }
        Steal::Retry => {
            panic!("Steal was interrupted, try again");
        }
    }
}

struct Transaction {
    id: u32,
    read_locks: Vec<u32>,
    write_locks: Vec<u32>,
}

impl Task for Transaction {
    type ResourceID = u32;

    fn read_locks(&self) -> &[Self::ResourceID] {
        &self.read_locks
    }

    fn write_locks(&self) -> &[Self::ResourceID] {
        &self.write_locks
    }
}
