# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Build all crates
cargo build

# Build in release mode
cargo build --release

# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p runtime-manager
cargo test -p transaction-runtime

# Run a single test by name
cargo test test_name

# Lint with clippy
cargo clippy

# Format code
cargo fmt
```

## Architecture Overview

KAS-L2 is a Layer 2 execution runtime implemented as a modular Rust workspace with generic, pluggable components.

### Workspace Structure

```
core/               Core utilities (atomics, macros)
runtime/            Execution runtime (manager, workers, state, types)
storage/            Persistence layer (manager, types, rocksdb-store)
transaction-runtime/  Transaction model (21 focused crates)
vm/                 VM implementations bridging to runtime
move-runtime/       Move language VM integration
```

### Key Architectural Concepts

**Generic Type Architecture**: The runtime is VM and storage agnostic. Components like `RuntimeManager<S: Store, V: VmInterface>` accept any implementation of the required traits.

**Smart Pointer Pattern**: The `#[smart_pointer]` macro in `core/macros` creates Arc-based wrappers with automatic Deref and cyclic reference support. Used for `RuntimeBatch`, `RuntimeTx`, `ResourceAccess`, `StateDiff`.

**Resource Dependency Chains**: Transactions accessing the same resource form linked chains enabling parallel execution of independent transactions while maintaining ordering for dependent ones.

**Three-Stage Batch Lifecycle**:
1. Processed - All transactions executed, effects computed
2. Persisted - All state diffs written to storage
3. Committed - Latest pointers updated

### Core Types

**runtime/manager**:
- `RuntimeManager` - Main entry point managing batches, workers, and storage
- `RuntimeBatch` - Batch of transactions with atomic coordination (pending counters, async latches)
- `RuntimeTx` - Individual transaction with resource access tracking
- `ResourceAccess` - Tracks a transaction's access to a resource with read/written state versions

**runtime/types**:
- `AccessMetadata` trait - Resource ID and access type (Read/Write)
- `Transaction` trait - Collection of accessed resources

**transaction-runtime/**:
- `Transaction` - Instructions and accessed objects
- `Instruction` - PublishProgram or CallProgram
- `ObjectId` - Enum: Empty, Program(Address), Data(Address)
- `Data` - Serializable data with owning_program, type_id, bytes
- `Lock` - Shared or SignatureLock controlling write access
- `AuthenticatedData` - Data with optional mutation capability

**vm/vm**:
- `VmInterface` trait - Contract for VM implementations with associated types for Transaction, Effects, ResourceId, etc.

### Concurrency Model

Lock-free coordination using:
- `AtomicU64` for pending counters
- `AtomicAsyncLatch` for async coordination across threads
- `AtomicOptionArc` for atomic state publishing
- Work-stealing execution via crossbeam deque

### Storage Model

- `Store` trait abstracts KV storage with `StateSpace` namespacing
- State diffs associated with batches, not individual transactions
- RocksDB backend relies on WAL for crash consistency
- Storage errors are treated as non-recoverable (panics)

### Serialization

Borsh is used throughout for deterministic serialization of State, Lock, Data, and Program types.