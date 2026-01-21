# node/

Defines **how we connect to the real world**. This layer provides the concrete VM implementation that integrates all other layers.

## Crates

### vm/
`vprogs-node-vm`

Reference implementation of the VmInterface trait:

```rust
pub struct VM { /* ... */ }

impl VmInterface for VM {
    type Transaction = Transaction;
    type TransactionEffects = TransactionEffects;
    type ResourceId = ObjectId;
    type AccessMetadata = ObjectAccess;
    type Error = VmError;

    fn process_transaction<S: Store>(
        &self,
        tx: &Self::Transaction,
        resources: &mut [AccessHandle<S, Self>],
    ) -> Result<Self::TransactionEffects, Self::Error> {
        // Uses TransactionRuntime from transaction-runtime layer
    }
}
```

This implementation:
- Bridges the abstract scheduling types to concrete transaction-runtime types
- Uses `TransactionRuntime` for actual transaction execution
- Handles program loading and capability management
- Produces `TransactionEffects` as execution results

## Layer Position

```
┌─────────────────────────────────────────┐
│  Layer 5: node  ◄── You are here        │
├─────────────────────────────────────────┤
│  Layer 4: transaction-runtime           │
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

The node layer is the top layer. It integrates all layers into a working system.

## Usage

```rust
use vprogs_node_vm::VM;
use vprogs_scheduling_scheduler::{Scheduler, ExecutionConfig, StorageConfig};

// Create the VM
let vm = VM::new();

// Create the scheduler with the VM
let scheduler = Scheduler::new(
    ExecutionConfig::new(num_workers, vm),
    StorageConfig::new(store),
);

// Schedule transactions
let batch = scheduler.schedule(transactions);

// Wait for completion
batch.wait_committed_blocking();
```

## Design Philosophy

The node layer:
1. Is the only layer that knows about all other layers
2. Provides the concrete types for the abstract traits
3. Can be replaced with alternative VM implementations
4. Serves as the integration point for the complete system
