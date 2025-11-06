use kas_l2_rocksdb_store::RocksDbStore;

use crate::Transaction;

pub type AccessHandle<'a> = kas_l2_runtime_core::AccessHandle<'a, RocksDbStore, Transaction>;
