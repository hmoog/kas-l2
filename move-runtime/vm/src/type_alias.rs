use kas_l2_storage_rocksdb_store::RocksDbStore;

use crate::Transaction;

pub type AccessHandle<'a> = kas_l2_runtime::AccessHandle<'a, RocksDbStore, Transaction>;
