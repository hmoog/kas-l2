use rocksdb::{Options, WriteOptions};
use tap::Tap;

pub trait Config: Send + Sync + 'static {
    fn db_opts() -> Options {
        Options::default().tap_mut(|o| {
            // Concurrency & throughput
            o.increase_parallelism(num_cpus::get() as i32);
            o.set_max_background_jobs(8);
            o.set_max_subcompactions(2);

            // Better write overlap and WAL throughput
            o.set_allow_concurrent_memtable_write(true);
            o.set_enable_pipelined_write(true);
            o.set_unordered_write(true); // WAL order preserved; memtable order may differ

            // Smooth out background I/O
            o.set_bytes_per_sync(1 << 20); // 1 MiB
            o.set_wal_bytes_per_sync(1 << 20);

            o.set_level_compaction_dynamic_level_bytes(true);
            o.set_paranoid_checks(true);
        })
    }

    fn write_opts() -> WriteOptions {
        WriteOptions::default().tap_mut(|o| {
            o.set_sync(false); // no fsync on each write (group commit FTW)
            o.disable_wal(false); // keep WAL (crash replay is our durability)
        })
    }

    fn cf_data_opts() -> Options {
        Options::default()
    }

    fn cf_latest_pointers_opts() -> Options {
        Options::default()
    }

    fn cf_old_pointers_opts() -> Options {
        Options::default()
    }

    fn cf_meta_opts() -> Options {
        Options::default()
    }
}

pub struct DefaultConfig;
impl Config for DefaultConfig {}
