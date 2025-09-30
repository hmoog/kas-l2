extern crate core;

use std::{collections::HashMap, thread::sleep, time::Duration};

use kas_l2_runtime_core::{AccessMetadata, AccessType, ResourceHandle, RuntimeBuilder};

#[test]
pub fn test_executor() {
    let mut runtime = RuntimeBuilder::default()
        .with_storage(KVStore(HashMap::new()))
        .with_transaction_processor(
            |tx: &Transaction, _resources: &mut [ResourceHandle<Transaction>]| {
                for r in _resources {
                    match r.access_metadata().access_type {
                        AccessType::Read => {
                            println!(
                                "Transaction {} reading from resource {}: {:?}",
                                tx.id,
                                r.access_metadata().resource_id(),
                                r.data()
                            );
                        }
                        AccessType::Write => {
                            println!(
                                "Transaction {} writing to resource {}: {:?}",
                                tx.id,
                                r.access_metadata().resource_id(),
                                r.data()
                            );
                        }
                    }
                }
                println!("Executing transaction with id {}", tx.id);
                sleep(tx.duration);
                println!("Finished transaction with id {}", tx.id);
            },
        )
        .with_batch_processor(|batch| {
            println!(
                "Processed batch with {} transactions",
                batch.scheduled_transactions().len()
            );
        })
        .build();

    runtime.process(vec![
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
    ]);

    runtime.process(vec![
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
    ]);

    sleep(Duration::from_secs(1));

    runtime.shutdown();
}

struct Transaction {
    id: u32,
    duration: Duration,
    access: Vec<Access>,
}

impl kas_l2_runtime_core::Transaction for Transaction {
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

impl kas_l2_runtime_core::AccessMetadata<u32> for Access {
    fn resource_id(&self) -> u32 {
        self.resource_id
    }

    fn access_type(&self) -> AccessType {
        self.access_type
    }
}

pub struct KVStore(HashMap<u32, Vec<u8>>);

impl kas_l2_runtime_core::Storage<u32> for KVStore {
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
