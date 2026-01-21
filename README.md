# vprogs

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/kaspanet/vprogs)

A Rust-based Layer 2 implementation for the Kaspa network, featuring a transaction scheduler, execution runtime, and storage management system.

## Architecture

vprogs is organized as a modular monorepo with clear separation of concerns across six domains. Each domain has a single responsibility and communicates with others through well-defined trait boundaries.

```
┌─────────────────────────────────────────────────────────────────┐
│                            node/                                │
│                   VM Reference Implementation                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────┐    ┌────────────────────────────────┐  │
│  │    scheduling/      │    │     transaction-runtime/       │  │
│  │                     │    │                                │  │
│  │  Batch Processing   │◄───│   Execution Semantics          │  │
│  │  Resource Tracking  │    │   Programs & Contexts          │  │
│  │  Parallel Execution │    │                                │  │
│  └──────────┬──────────┘    └────────────────────────────────┘  │
│             │                                                   │
│             ▼                                                   │
│  ┌─────────────────────┐    ┌────────────────────────────────┐  │
│  │      storage/       │◄───│           state/               │  │
│  │                     │    │                                │  │
│  │  Persistence Layer  │    │   State Definitions            │  │
│  │  Read/Write Coords  │    │   Versioning & Pointers        │  │
│  └─────────────────────┘    └────────────────────────────────┘  │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                            core/                                │
│                    Foundation Utilities                         │
└─────────────────────────────────────────────────────────────────┘
```

## Domains

| Domain | Purpose | Documentation |
|--------|---------|---------------|
| [core/](core/) | Foundation utilities: atomics, macros | [README](core/README.md) |
| [state/](state/) | State definitions: what we store | [README](state/README.md) |
| [storage/](storage/) | Persistence layer: how we store | [README](storage/README.md) |
| [scheduling/](scheduling/) | Execution orchestration: how we provide access | [README](scheduling/README.md) |
| [transaction-runtime/](transaction-runtime/) | Execution semantics: what we do with access | [README](transaction-runtime/README.md) |
| [node/](node/) | VM implementation: how we connect to the real world | [README](node/README.md) |

## Design Principles

### Layered Architecture

Each domain builds on the layers below it:

1. **core** - Zero dependencies on other domains
2. **state** - Defines state spaces and versioning semantics
3. **storage** - Implements persistence using state definitions
4. **scheduling** - Orchestrates execution using storage
5. **transaction-runtime** - Defines execution semantics
6. **node** - Integrates everything into a concrete VM

### Trait-Driven Extensibility

Core abstractions are defined as traits, allowing different implementations:

- `VmInterface` - Abstract transaction processor
- `Store` / `ReadStore` / `WriteBatch` - Abstract state persistence
- `ResourceId` / `Transaction` / `AccessMetadata` - Abstract scheduling types

### Batch-Oriented Execution

Transactions are grouped into batches for atomic processing:

- `RuntimeBatch` - Groups transactions for execution
- `StateDiff` - Captures state changes per resource per batch
- `Rollback` - Reverts state changes when needed

## Build Commands

```bash
cargo build                 # Debug build
cargo build --release       # Release build
cargo test                  # Run all tests
cargo test --test e2e       # Run integration tests only
cargo fmt                   # Format code
cargo clippy                # Lint code
```

## Package Naming Convention

All packages follow the pattern: `vprogs-{domain}-{crate}`

Examples:
- `vprogs-scheduling-scheduler`
- `vprogs-storage-manager`
- `vprogs-state-versioned-state`

## Serialization

Uses [Borsh](https://borsh.io/) for state serialization throughout the codebase.

## License

See [LICENSE](LICENSE) for details.
