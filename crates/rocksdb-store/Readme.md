# RocksDB Store

This crate provides a RocksDB-based implementation of a KV Store compatible with the runtime-core crate.                                         

## Design Principles

- storage errors are treated as non-recoverable errors (we assume the local storage to be reliable)
- we accordingly panic in place to prevent any further corruption of the underlying database
- we rely on eventual consistency and operate without any artificial fsync or wal flush boundaries
- we rely on RocksDB's own crash consistency mechanisms (WAL replay, atomic writes) to ensure data integrity
- we use a simple namespace abstraction to separate different categories of data within the same RocksDB instance