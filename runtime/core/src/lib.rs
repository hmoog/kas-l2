mod runtime;
mod vm;

pub(crate) mod data {
    pub(crate) mod state_diff;
}

pub(crate) mod runtime_tx;

pub(crate) mod notarization {
    pub(crate) mod notarization_worker;
}

pub(crate) mod storage {
    pub(crate) mod cmd;
}

pub(crate) mod resources {
    pub(crate) mod access_handle;
    pub(crate) mod resource;
    pub(crate) mod resource_access;
}

pub(crate) mod scheduling {
    pub(crate) mod batch;
    pub(crate) mod scheduler;
}

pub use crate::{
    data::state_diff::{StateDiff, StateDiffRef},
    resources::access_handle::AccessHandle,
    runtime::Runtime,
    runtime_tx::RuntimeTx,
    scheduling::batch::{Batch, BatchRef},
    vm::VM,
};

pub(crate) use crate::{
    notarization::notarization_worker::NotarizationWorker,
    resources::{resource::Resource, resource_access::ResourceAccess},
    runtime_tx::RuntimeTxRef,
    scheduling::scheduler::Scheduler,
    storage::cmd::{Read, Write},
};

pub(crate) type Executor<S, V> =
    kas_l2_runtime_execution::Executor<RuntimeTx<S, V>, Batch<S, V>, V>;

pub(crate) type WorkersApi<S, V> =
    kas_l2_runtime_execution::WorkersApi<RuntimeTx<S, V>, Batch<S, V>, V>;

pub(crate) type Worker<S, V> = kas_l2_runtime_execution::Worker<RuntimeTx<S, V>, Batch<S, V>, V>;

pub(crate) type BatchQueue<S, V> =
    kas_l2_runtime_execution::BatchQueue<Batch<S, V>, RuntimeTx<S, V>>;
