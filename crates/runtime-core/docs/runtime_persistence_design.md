# DLT Runtime: State Persistence & Versioned Storage Design

## 1. Motivation

The runtime executes transactions in parallel inside a lock-free DAG.  
Intermediate per-transaction state lives only in memory; only **aggregate state changes at batch boundaries** are persisted.  
This design minimizes I/O latency and allows the runtime to scale linearly with available CPU cores.

Persistence must therefore provide:

- atomic visibility of completed batches,
- crash tolerance without explicit fsync calls, and
- efficient parallel I/O.

---

## 2. Architecture Overview

The runtime is divided into two planes:

| Plane | Responsibility | Notes |
|--------|----------------|-------|
| **Execution plane** | Executes transactions concurrently, tracks fine-grained resource dependencies in a DAG. | Purely in-memory, lock-free. |
| **Persistence plane** | Persists aggregated `StateDiff`s at batch boundaries, provides atomic visibility and rollback. | Asynchronous write workers. |

```
DAG execution ─► StateDiff aggregation ─► Write queue ─► RocksDB (batch commit)
```

Only *final* batch results are written to disk.  
Intermediate states disappear when the DAG is dropped.

---

## 3. Pointer-Flip Commit Primitive

The key persistence primitive is the **pointer flip**:

1. Each resource has immutable versioned state entries:

   ```
   (resource_id, version_id) → serialized_state
   ```

2. A separate pointer map records which version is “current”:

   ```
   resource_id → version_id
   ```

3. When a new batch finishes executing:
    - all `(resource_id, new_version)` values are written first,
    - then a single **atomic WriteBatch** updates the pointer map.

   ```
   Before:
       id=42 → version=100
   After pointer flip:
       id=42 → version=101
   ```

That atomic pointer update makes the new version visible instantly.  
The old version can immediately be garbage-collected.

### Crash semantics

| Crash timing | Result |
|---------------|---------|
| Before pointer flip | Old state remains visible |
| After pointer flip | New state visible and consistent |
| During write of new states | New versions orphaned but never referenced |

The WAL ensures ordering; no partial visibility is possible.

---

## 4. Data Model

Each logical data type lives in its own namespace (mapped to a RocksDB **column family**):

| Namespace | Contents | Access pattern |
|------------|-----------|----------------|
| **StateData** | `(resource_id, version_id) → state` | Bulk sequential writes |
| **LatestPointers** | `resource_id → version_id` | Small random updates |
| **Diffs** | `(batch_id, resource_id) → StateDiff` | Append-only, rollback support |
| **Meta** | runtime metadata | low frequency |

Column families allow independent tuning and compaction while keeping atomic multi-CF commits.

---

## 5. Write Path

1. **Execution:** DAG produces new `StateDiff`s for each batch.
2. **Queueing:** Diffs are enqueued for persistence.
3. **Write workers:**
    - 2–4 threads per RocksDB instance.
    - Drain global queues, aggregate writes across all CFs into large `WriteBatch`es.
    - Submit asynchronously to RocksDB (`sync = false`).
4. **Commit coordinator:**
    - Waits until all writes for batch *N* are acknowledged.
    - Issues one atomic `WriteBatch` to update `LatestPointers`.
    - Declares batch *N* final (“pointer flip”).

This approach maximizes WAL throughput and ensures causal ordering.

---

## 6. Durability Model

### No explicit WAL flushes

The runtime never calls `flush_wal(true)` manually.

**Reasoning**

- Each RocksDB write appends to the WAL before returning; replay restores all complete batches.
- Explicit fsync boundaries (one per commit) add 0.1–2 ms latency per call and prevent group commits.
- The runtime can tolerate losing the *last uncommitted* batch; consensus will replay it.

**Invariant**

> All data for a batch must be written before the pointer-flip batch is issued.

If that ordering is preserved, RocksDB’s own WAL replay guarantees crash safety.

---

## 7. Crash and Reorg Handling

- **Crash:**  
  RocksDB replays the WAL to the last consistent sequence.  
  Old pointers remain valid; unreferenced states are ignored.

- **Reorg:**  
  `StateDiff`s contain both the read and written states.  
  A rollback re-applies the old (read) states and resets pointers.  
  Once finalized, old diffs can be deleted.

---

## 8. Key-Value Store Abstraction

To hide RocksDB details, the runtime depends on a minimal generic interface:

```rust
pub enum Namespace {
    Default,
    StateData,
    LatestPointers,
    Diffs,
    Meta,
}

pub trait KeyValueStore {
    type Batch<'a>: WriteBatchOps<'a>;

    fn get(&self, ns: Namespace, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn put(&self, ns: Namespace, key: &[u8], value: &[u8]) -> Result<()>;
    fn delete(&self, ns: Namespace, key: &[u8]) -> Result<()>;
    fn batch(&self) -> Self::Batch<'_>;
    fn write_batch(&self, batch: Self::Batch<'_>) -> Result<()>;
}

pub trait WriteBatchOps<'a> {
    fn put(&mut self, ns: Namespace, key: &[u8], value: &[u8]);
    fn delete(&mut self, ns: Namespace, key: &[u8]);
}
```

This maps directly onto RocksDB column families but stays backend-agnostic.  
No durability or flush controls are exposed; causal ordering is enforced by the runtime.

---

## 9. Performance Characteristics

| Mechanism | Benefit |
|------------|----------|
| In-memory DAG | Lock-free parallel execution |
| Batch-level persistence | Minimal write amplification |
| Asynchronous write workers | Continuous WAL utilization |
| Mixed-CF batches | Single WAL append per batch |
| No explicit fsyncs | Avoids I/O fences, uses RocksDB group commit |
| Causal pointer flip | Atomic visibility, crash-safe |

On NVMe hardware this design can sustain **hundreds of thousands of state updates per second** with consistent crash recovery.

---

## 10. Summary

- **Correctness:** guaranteed by *causal ordering* and *atomic pointer flip*.
- **Crash safety:** inherited from RocksDB WAL replay; no fsync stalls.
- **Throughput:** maximized by asynchronous batching and mixed-CF writes.
- **Simplicity:** exposed through a minimal namespaced KV interface.

**In short:**
> The runtime achieves atomic, crash-tolerant, and high-throughput state persistence without explicit WAL synchronization by ordering all writes before a single atomic pointer-flip commit.
