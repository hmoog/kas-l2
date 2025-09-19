use std::sync::Arc;

use crate::ResourceAccess;

pub trait GuardConsumer: Sized {
    type ConsumerGuardID;
    fn notify(self: &Arc<Self>, guard: Arc<ResourceAccess<Self>>);
}
