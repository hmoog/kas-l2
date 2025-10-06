mod runtime;
mod runtime_batch_processor;
mod runtime_builder;

pub(crate) mod execution {
    pub(crate) mod executor;
    pub(crate) mod runtime_tx;
    pub(crate) mod worker;
    pub(crate) mod workers_api;
}

pub(crate) mod resources {
    pub(crate) mod access_handle;
    pub(crate) mod access_type;
    pub(crate) mod resource;
    pub(crate) mod resource_access;
    pub(crate) mod resource_provider;
    pub(crate) mod state;
    pub(crate) mod state_diff;
}

pub(crate) mod scheduling {
    pub(crate) mod batch;
    pub(crate) mod batch_api;
    pub(crate) mod pending_batches;
    pub(crate) mod scheduler;
}

pub(crate) mod traits {
    pub(crate) mod access_metadata;
    pub(crate) mod batch_processor;
    pub(crate) mod resource_id;
    pub(crate) mod storage;
    pub(crate) mod transaction;
    pub(crate) mod transaction_processor;
}

pub(crate) mod utils {
    pub(crate) mod vec_ext;
}

pub use crate::{
    execution::runtime_tx::RuntimeTx,
    resources::{
        access_handle::AccessHandle,
        access_type::AccessType,
        state::State,
        state_diff::{StateDiff, StateDiffRef},
    },
    runtime::Runtime,
    runtime_builder::RuntimeBuilder,
    scheduling::{
        batch::Batch,
        batch_api::{BatchApi, BatchApiRef},
    },
    traits::{
        access_metadata::AccessMetadata, batch_processor::BatchProcessor, resource_id::ResourceId,
        storage::Storage, transaction::Transaction, transaction_processor::TransactionProcessor,
    },
};
pub(crate) use crate::{
    execution::{
        executor::Executor, runtime_tx::RuntimeTxRef, worker::Worker, workers_api::WorkersApi,
    },
    resources::{
        resource::Resource, resource_access::ResourceAccess, resource_provider::ResourceProvider,
    },
    runtime_batch_processor::RuntimeBatchProcessor,
    scheduling::{pending_batches::PendingBatches, scheduler::Scheduler},
    utils::vec_ext::VecExt,
};
