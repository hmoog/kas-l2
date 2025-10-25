mod builder;
mod runtime;
pub(crate) mod utils;

pub(crate) mod data {
    pub(crate) mod state;
    pub(crate) mod state_diff;
    pub(crate) mod versioned_state;
}

pub(crate) mod execution {
    pub(crate) mod executor;
    pub(crate) mod runtime_tx;
    pub(crate) mod transaction;
    pub(crate) mod transaction_processor;
    pub(crate) mod worker;
    pub(crate) mod workers_api;
}

pub(crate) mod storage {
    pub(crate) mod read_cmd;
    pub(crate) mod runtime_state;
    pub(crate) mod write_cmd;
}

pub(crate) mod resources {
    pub(crate) mod access_handle;
    pub(crate) mod access_metadata;
    pub(crate) mod access_type;
    pub(crate) mod resource;
    pub(crate) mod resource_access;
    pub(crate) mod resource_id;
}

pub(crate) mod scheduling {
    pub(crate) mod batch;
    pub(crate) mod batch_post_processor;
    pub(crate) mod batch_processor;
    pub(crate) mod batch_queue;
    pub(crate) mod scheduler;
}

pub use crate::{
    builder::RuntimeBuilder,
    data::{
        state::State,
        state_diff::{StateDiff, StateDiffRef},
        versioned_state::VersionedState,
    },
    execution::{
        runtime_tx::RuntimeTx, transaction::Transaction,
        transaction_processor::TransactionProcessor,
    },
    resources::{
        access_handle::AccessHandle, access_metadata::AccessMetadata, access_type::AccessType,
        resource_id::ResourceId,
    },
    runtime::Runtime,
    scheduling::{
        batch::{Batch, BatchRef},
        batch_post_processor::BatchPostProcessor,
    },
    storage::runtime_state::RuntimeState,
};
pub(crate) use crate::{
    execution::{
        executor::Executor, runtime_tx::RuntimeTxRef, worker::Worker, workers_api::WorkersApi,
    },
    resources::{resource::Resource, resource_access::ResourceAccess},
    scheduling::{batch_processor::BatchProcessor, batch_queue::BatchQueue, scheduler::Scheduler},
    utils::VecExt,
};
