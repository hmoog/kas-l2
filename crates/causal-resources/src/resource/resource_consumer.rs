use std::sync::Arc;
use kas_l2_core::Transaction;
use crate::resource::ResourceProvider;

pub trait ResourceConsumer<T: Transaction>: Sized {
    type ResourceID;
    fn notify(self: &Arc<Self>, resource_provider: Arc<ResourceProvider<T, Self>>);
}
