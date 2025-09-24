use std::sync::Arc;

pub trait AtomicAccessor {
    fn available(self: &Arc<Self>);
}
