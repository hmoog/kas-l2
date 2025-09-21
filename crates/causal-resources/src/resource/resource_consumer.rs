use std::sync::Arc;

use crate::resource::ResourceProvider;

pub trait ResourceConsumer: Sized {
    type ResourceID;
    fn notify(self: &Arc<Self>, resource_provider: Arc<ResourceProvider<Self>>);
}
