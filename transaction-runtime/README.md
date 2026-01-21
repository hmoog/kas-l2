# transaction-runtime/

Defines **what we do with access**. This layer specifies the execution semantics for transactions, including programs, contexts, and effects.

## Crates

The transaction-runtime is decomposed into fine-grained crates for modularity:

### Core Types

| Crate | Package | Purpose |
|-------|---------|---------|
| address/ | `vprogs-transaction-runtime-address` | Account and contract addresses |
| pubkey/ | `vprogs-transaction-runtime-pubkey` | Public key representation |
| object-id/ | `vprogs-transaction-runtime-object-id` | Unique object identifiers |
| instruction/ | `vprogs-transaction-runtime-instruction` | Transaction instructions |
| transaction/ | `vprogs-transaction-runtime-transaction` | Transaction structure |
| transaction-effects/ | `vprogs-transaction-runtime-transaction-effects` | Execution results |
| error/ | `vprogs-transaction-runtime-error` | Error types and VmResult<T> |

### Data & Access

| Crate | Package | Purpose |
|-------|---------|---------|
| data/ | `vprogs-transaction-runtime-data` | Raw data representation |
| authenticated-data/ | `vprogs-transaction-runtime-authenticated-data` | Data with capability proofs |
| object-access/ | `vprogs-transaction-runtime-object-access` | Object access descriptors |
| lock/ | `vprogs-transaction-runtime-lock` | Resource locking mechanisms |
| signature-lock/ | `vprogs-transaction-runtime-signature-lock` | Signature-based locks |

### Execution Contexts

| Crate | Package | Purpose |
|-------|---------|---------|
| auth-context/ | `vprogs-transaction-runtime-auth-context` | Authentication context |
| data-context/ | `vprogs-transaction-runtime-data-context` | Data access context |
| program-context/ | `vprogs-transaction-runtime-program-context` | Program execution context |

### Programs

| Crate | Package | Purpose |
|-------|---------|---------|
| program/ | `vprogs-transaction-runtime-program` | Program definition |
| program-type/ | `vprogs-transaction-runtime-program-type` | Program type classification |
| program-arg/ | `vprogs-transaction-runtime-program-arg` | Program arguments |
| builtin-capabilities/ | `vprogs-transaction-runtime-builtin-capabilities` | Built-in program capabilities |

### Integration

| Crate | Package | Purpose |
|-------|---------|---------|
| transaction-runtime/ | `vprogs-transaction-runtime` | Main runtime integration |

## Layer Position

```
┌─────────────────────────────────────────┐
│  Layer 5: node                          │
├─────────────────────────────────────────┤
│  Layer 4: transaction-runtime           │
│           ◄── You are here              │
├─────────────────────────────────────────┤
│  Layer 3: scheduling                    │
├─────────────────────────────────────────┤
│  Layer 2: state                         │
├─────────────────────────────────────────┤
│  Layer 1: storage                       │
├─────────────────────────────────────────┤
│  Layer 0: core                          │
└─────────────────────────────────────────┘
```

The transaction-runtime layer defines execution semantics. It is used by the node layer to implement the VmInterface trait.

## Design Philosophy

The fine-grained crate structure:
1. Minimizes coupling between concepts
2. Enables incremental compilation
3. Makes dependencies explicit
4. Allows selective reuse of components

Each crate has a single responsibility, following the Unix philosophy of doing one thing well.
