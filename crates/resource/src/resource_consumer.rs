use std::sync::Arc;

use crate::ResourceAccess;

pub trait ResourceConsumer: Sized {
    type ResourceID;
    fn notify(self: &Arc<Self>, guard: Arc<ResourceAccess<Self>>);
}
