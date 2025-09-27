use std::sync::Arc;

pub trait Consumer {
    fn resources_available(self: &Arc<Self>);
}
