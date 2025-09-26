extern crate core;

use std::{collections::HashMap, thread::sleep, time::Duration};

use kas_l2_core::resources::{AccessHandle, AccessType};
use kas_l2_executor::Executor;
use kas_l2_scheduler::{ResourcesManager, Scheduler};

#[test]
pub fn test_executor() {
    let resources_manager = ResourcesManager::new(KVStore(HashMap::new()));
    let mut scheduler = Scheduler::new(resources_manager);

    let executor = Executor::new(
        4,
        |tx: &Transaction, _resources: &mut [AccessHandle<Transaction>]| {
            println!("Executing transaction with id {}", tx.id);
            sleep(tx.duration);
            println!("Finished transaction with id {}", tx.id);
        },
    );

    executor.execute(scheduler.schedule(vec![
        Transaction {
            id: 0,
            duration: Duration::from_millis(10),
            access: vec![
                Access {
                    resource_id: 1,
                    access_type: AccessType::Write,
                },
                Access {
                    resource_id: 3,
                    access_type: AccessType::Read,
                },
            ],
        },
        Transaction {
            id: 1,
            duration: Duration::from_millis(1),
            access: vec![
                Access {
                    resource_id: 1,
                    access_type: AccessType::Write,
                },
                Access {
                    resource_id: 2,
                    access_type: AccessType::Write,
                },
            ],
        },
        Transaction {
            id: 2,
            duration: Duration::from_millis(1),
            access: vec![Access {
                resource_id: 3,
                access_type: AccessType::Read,
            }],
        },
    ]));

    executor.execute(scheduler.schedule(vec![
        Transaction {
            id: 3,
            duration: Duration::from_millis(1),
            access: vec![
                Access {
                    resource_id: 1,
                    access_type: AccessType::Write,
                },
                Access {
                    resource_id: 3,
                    access_type: AccessType::Read,
                },
            ],
        },
        Transaction {
            id: 4,
            duration: Duration::from_millis(1),
            access: vec![
                Access {
                    resource_id: 10,
                    access_type: AccessType::Write,
                },
                Access {
                    resource_id: 20,
                    access_type: AccessType::Write,
                },
            ],
        },
    ]));

    sleep(Duration::from_secs(1));

    executor.shutdown();
}

struct Transaction {
    id: u32,
    duration: Duration,
    access: Vec<Access>,
}

impl kas_l2_core::transactions::Transaction for Transaction {
    type ResourceID = u32;

    type AccessMetadata = Access;

    fn accessed_resources(&self) -> &[Self::AccessMetadata] {
        &self.access
    }
}

#[derive(Clone)]
struct Access {
    resource_id: u32,
    access_type: AccessType,
}

impl kas_l2_core::resources::AccessMetadata<u32> for Access {
    fn resource_id(&self) -> u32 {
        self.resource_id
    }

    fn access_type(&self) -> AccessType {
        self.access_type
    }
}

pub struct KVStore(HashMap<u32, Vec<u8>>);

impl kas_l2_core::storage::KvStore<u32> for KVStore {
    type Error = std::io::Error;

    fn get(&self, key: &u32) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(self.0.get(key).cloned())
    }

    fn put(&mut self, key: u32, value: Vec<u8>) -> Result<(), Self::Error> {
        self.0.insert(key, value);
        Ok(())
    }
    fn delete(&mut self, key: &u32) -> Result<bool, Self::Error> {
        Ok(self.0.remove(key).is_some())
    }
}
