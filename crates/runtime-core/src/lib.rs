pub mod storage;
pub mod transaction;

mod access_handle;
mod access_metadata;
mod access_type;
mod batch;
mod batch_api;
mod consumer;
mod resource;
mod resource_id;
mod resource_manager;
mod resource_provider;
mod resources;
mod scheduled_transaction;
mod scheduler;
mod state;
mod transaction_processor;

pub use access_handle::AccessHandle;
pub use access_metadata::AccessMetadata;
pub use access_type::AccessType;
pub use batch::Batch;
pub use batch_api::BatchAPI;
pub use consumer::Consumer;
pub use resource_id::ResourceID;
pub use resources::Resources;
pub use scheduled_transaction::ScheduledTransaction;
pub use scheduler::Scheduler;
pub use state::State;
pub use transaction::Transaction;
pub use transaction_processor::TransactionProcessor;

/// A resource provider specialized for `ScheduledTransaction<T>`.
pub type ResourceProvider<T, K> =
    resource_provider::ResourceProvider<T, ScheduledTransaction<T>, K>;
