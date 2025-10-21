use std::{marker::PhantomData, path::Path, sync::Arc};

use kas_l2_runtime_core::RuntimeState;
use kas_l2_storage::Store;
use rocksdb::DB;

use crate::{
    config::{Config, DefaultConfig},
    runtime_state_ext::RuntimeStateExt,
    write_batch::WriteBatch,
};

#[derive(Clone)]
pub struct RocksDbStore<C: Config = DefaultConfig> {
    db: Arc<DB>,
    write_opts: Arc<rocksdb::WriteOptions>,
    _marker: PhantomData<C>,
}

impl<C: Config> RocksDbStore<C> {
    pub fn open<P: AsRef<Path>>(path: P) -> Self {
        let mut db_opts = C::db_opts();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);

        Self {
            db: Arc::new(
                match DB::open_cf_descriptors(
                    &db_opts,
                    path,
                    <RuntimeState as RuntimeStateExt<C>>::all_descriptors(),
                ) {
                    Ok(db) => db,
                    Err(e) => panic!("failed to open RocksDB: {e}"),
                },
            ),
            write_opts: Arc::new(C::write_opts()),
            _marker: PhantomData,
        }
    }

    fn cf(&self, ns: &RuntimeState) -> &rocksdb::ColumnFamily {
        let cf_name = <RuntimeState as RuntimeStateExt<C>>::cf_name;
        match self.db.cf_handle(cf_name(ns)) {
            Some(cf) => cf,
            None => panic!("missing column family '{}'", cf_name(ns)),
        }
    }
}

impl<C: Config> Store for RocksDbStore<C> {
    type StateSpace = RuntimeState;
    type WriteBatch = WriteBatch<C>;

    fn get(&self, state_space: RuntimeState, key: &[u8]) -> Option<Vec<u8>> {
        match self.db.get_cf(self.cf(&state_space), key) {
            Ok(res) => res,
            Err(e) => panic!("rocksdb get failed: {e}"),
        }
    }

    fn put(&self, state_space: RuntimeState, key: &[u8], value: &[u8]) {
        if let Err(err) = self.db.put_cf(self.cf(&state_space), key, value) {
            panic!("rocksdb put failed: {err}");
        }
    }

    fn delete(&self, state_space: RuntimeState, key: &[u8]) {
        if let Err(err) = self.db.delete_cf(self.cf(&state_space), key) {
            panic!("rocksdb delete failed: {err}");
        }
    }

    fn write_batch(&self) -> WriteBatch<C> {
        WriteBatch::new(self.db.clone())
    }

    fn commit(&self, write_batch: WriteBatch<C>) {
        if let Err(err) = self.db.write_opt(write_batch.into(), &self.write_opts) {
            panic!("rocksdb write-batch commit failed: {err}");
        }
    }
}
