# core/

Foundation utilities used throughout the vprogs codebase. This domain has zero dependencies on other vprogs domains.

## Crates

### atomics/
`vprogs-core-atomics`

Concurrent atomic wrappers for lock-free programming:

- **AtomicArc** - Atomic reference-counted pointer
- **AtomicAsyncLatch** - Async-aware latch for signaling completion
- **AtomicEnum** - Atomic enum wrapper using discriminant mapping

### macros/
`vprogs-core-macros`

Procedural macros for code generation:

- **#[smart_pointer]** - Generates a wrapper struct with `Deref` implementation and a `Ref` type alias for `Arc<Data>` patterns

## Layer Position

```
┌─────────────────────────────────────────┐
│  scheduling / transaction-runtime       │
├─────────────────────────────────────────┤
│  storage / state                        │
├─────────────────────────────────────────┤
│  core  ◄── You are here                 │
└─────────────────────────────────────────┘
```

The core domain is the foundation layer. All other domains may depend on core, but core depends on no other vprogs domains.
