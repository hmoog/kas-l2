use std::sync::Arc;

use crate::ResourceAccess;

pub trait ResourceConsumer: Sized {
    type ConsumerGuardID;
    fn notify(self: &Arc<Self>, guard: Arc<ResourceAccess<Self>>);
}
