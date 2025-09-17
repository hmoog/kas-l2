use std::sync::Arc;

pub trait Consumer {
    fn notify(self: &Arc<Self>);
}
