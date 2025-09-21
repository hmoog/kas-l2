mod batch;
mod batch_api;
mod scheduled_transaction;
mod scheduler;

pub use batch::Batch;
pub use batch_api::BatchAPI;
use kas_l2_core::Transaction;
pub use scheduled_transaction::ScheduledTransaction;
pub use scheduler::Scheduler;

/// A resource provider specialized for `ScheduledTransaction<T>`.
pub type ResourcesManager<T> = kas_l2_causal_resources::ResourceManager<
    <T as Transaction>::ResourceID,
    ScheduledTransaction<T>,
>;
