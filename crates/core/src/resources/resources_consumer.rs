use std::sync::Arc;

pub trait ResourcesConsumer {
    fn resources_available(self: &Arc<Self>);
}
