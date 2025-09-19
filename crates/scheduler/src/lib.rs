mod batch;
mod batch_api;
mod scheduled_task;
mod scheduler;
mod task;

pub use batch::Batch;
pub use batch_api::BatchAPI;
pub use scheduled_task::ScheduledTask;
pub use scheduler::Scheduler;
pub use task::Task;

pub type ResourceProvider<T> =
    kas_l2_resource_provider::ResourceProvider<<T as Task>::ResourceID, ScheduledTask<T>>;
