extern crate core;

use std::{thread::sleep, time::Duration};

use kas_l2_causal_executor::Executor;
use kas_l2_causal_scheduler::{Scheduler, Task};

#[test]
pub fn test_executor() {
    let scheduler = Scheduler::<Transaction>::new();
    let batch1 = scheduler.schedule(vec![
        Transaction {
            id: 0,
            duration: Duration::from_millis(10),
            write_locks: vec![1],
            read_locks: vec![3],
        },
        Transaction {
            id: 1,
            duration: Duration::from_millis(1),
            write_locks: vec![1, 2],
            read_locks: vec![],
        },
        Transaction {
            id: 2,
            duration: Duration::from_millis(1),
            write_locks: vec![],
            read_locks: vec![3],
        },
    ]);
    let batch2 = scheduler.schedule(vec![
        Transaction {
            id: 3,
            duration: Duration::from_millis(1),
            write_locks: vec![1],
            read_locks: vec![3],
        },
        Transaction {
            id: 4,
            duration: Duration::from_millis(1),
            write_locks: vec![10, 20],
            read_locks: vec![],
        },
    ]);

    let executor = Executor::new(4, |tx: &Transaction| {
        println!("Executing transaction with id {}", tx.id);
        sleep(tx.duration);
        println!("Finished transaction with id {}", tx.id);
    });
    executor.execute(batch1);
    executor.execute(batch2);

    sleep(Duration::from_secs(1));

    executor.shutdown();
}

struct Transaction {
    id: u32,
    duration: Duration,
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
