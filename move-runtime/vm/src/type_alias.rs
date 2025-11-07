use kas_l2_storage_rocksdb_store::RocksDbStore;

use crate::VM;

pub type AccessHandle<'a> = kas_l2_runtime_core::AccessHandle<'a, RocksDbStore, VM>;
