use kas_l2_causal_executor::Executor;
use kas_l2_causal_scheduler::{Scheduler, Task};

#[test]
pub fn test_executor() {
    let executor = Executor::new(1, |tx: &Transaction| {
        println!("Executing task with id {}", tx.id);
    });

    executor.execute(Scheduler::schedule(vec![
        Transaction {
            id: 0,
            write_locks: vec![1],
            read_locks: vec![3],
        },
        Transaction {
            id: 1,
            write_locks: vec![1, 2],
            read_locks: vec![],
        },
        Transaction {
            id: 2,
            write_locks: vec![],
            read_locks: vec![3],
        },
    ]));

    std::thread::sleep(std::time::Duration::from_secs(1));

    executor.shutdown();
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
