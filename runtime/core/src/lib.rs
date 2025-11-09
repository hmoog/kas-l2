mod runtime;
mod vm;

pub(crate) mod data {
    pub(crate) mod state_diff;
}

pub(crate) mod execution {
    pub(crate) mod runtime_tx;
}

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
    pub(crate) mod batch_queue;
    pub(crate) mod scheduler;
}

pub use crate::{
    data::state_diff::{StateDiff, StateDiffRef},
    execution::runtime_tx::RuntimeTx,
    resources::access_handle::AccessHandle,
    runtime::Runtime,
    scheduling::batch::{Batch, BatchRef},
    vm::VM,
};
pub(crate) use crate::{
    execution::runtime_tx::RuntimeTxRef,
    notarization::notarization_worker::NotarizationWorker,
    resources::{resource::Resource, resource_access::ResourceAccess},
    scheduling::scheduler::Scheduler,
    storage::cmd::{Read, Write},
};

pub(crate) type Executor<S, V> = kas_l2_runtime_execution::Executor<
    V,
    execution::runtime_tx::RuntimeTx<S, V>,
    scheduling::batch_queue::BatchQueue<S, V>,
>;
