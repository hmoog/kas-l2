use std::sync::Arc;

pub trait GuardConsumer {
    type GuardID;
    fn notify(self: &Arc<Self>, guard_id: &Self::GuardID);
}
