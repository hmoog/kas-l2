use std::sync::Arc;

pub trait AtomicAccessor {
    fn notify(self: &Arc<Self>);
}
