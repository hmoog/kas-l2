# state/

Defines **what** we store. This layer establishes the semantic meaning of state without concern for how it is persisted.

## Crates

### space/
`vprogs-state-space`

Defines the logical partitions of state:

```rust
pub enum StateSpace {
    StateVersion,      // Versioned resource data
    StatePtrLatest,    // Points to current version of each resource
    StatePtrRollback,  // Points to previous version for rollback support
    Metadata,          // Metadata storage
}
```

### ptr-latest/
`vprogs-state-ptr-latest`

Type-safe operations for the StatePtrLatest column family:

- **Key**: `resource_id.to_bytes()`
- **Value**: `version.to_be_bytes()` (u64)

Provides `get`, `put`, `delete` operations with proper type constraints.

### ptr-rollback/
`vprogs-state-ptr-rollback`

Type-safe operations for the StatePtrRollback column family:

- **Key**: `batch_index.to_be_bytes() || resource_id.to_bytes()`
- **Value**: `old_version.to_be_bytes()` (u64)

Provides `put`, `delete`, and `iter_batch` for rollback operations.

### version/
`vprogs-state-version`

The main state abstraction combining all pointer operations:

```rust
pub struct StateVersion<R: ResourceId> {
    resource_id: R,
    version: u64,
    data: Vec<u8>,
}
```

Key operations:
- `from_latest_data()` - Load current state from store
- `write_data()` - Persist versioned data
- `write_latest_ptr()` - Update the current version pointer
- `write_rollback_ptr()` - Record previous version for rollback

## Layer Position

```
┌─────────────────────────────────────────┐
│  Layer 3: scheduling                    │
├─────────────────────────────────────────┤
│  Layer 2: state  ◄── You are here       │
├─────────────────────────────────────────┤
│  Layer 1: storage                       │
├─────────────────────────────────────────┤
│  Layer 0: core                          │
└─────────────────────────────────────────┘
```

The state layer defines the data model. The storage layer below implements persistence using these definitions. The scheduling layer above uses state abstractions to track resource versions.

## Design Philosophy

State is separated from storage to:
1. Keep data semantics independent of persistence mechanism
2. Allow the scheduling layer to reason about state without knowing storage details
3. Enable different storage backends without changing state definitions
