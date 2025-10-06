use std::sync::Arc;

use kas_l2_atomic::AtomicOptionArc;
use kas_l2_runtime_macros::smart_pointer;

use crate::{State, Transaction};

#[smart_pointer]
pub struct StateDiff<T: Transaction> {
    resource_id: T::ResourceId,
    read_state: AtomicOptionArc<State<T>>,
    written_state: AtomicOptionArc<State<T>>,
}

impl<T: Transaction> StateDiff<T> {
    pub fn new(resource_id: T::ResourceId) -> Self {
        Self(Arc::new(StateDiffData {
            resource_id,
            read_state: AtomicOptionArc::empty(),
            written_state: AtomicOptionArc::empty(),
        }))
    }

    pub fn resource_id(&self) -> &T::ResourceId {
        &self.resource_id
    }

    pub fn read_state(&self) -> Arc<State<T>> {
        self.read_state.load().expect("read state unknown")
    }

    pub fn written_state(&self) -> Arc<State<T>> {
        self.written_state.load().expect("written state unknown")
    }

    pub(crate) fn set_read_state(&self, state: Arc<State<T>>) {
        self.read_state.store(Some(state))
    }

    pub(crate) fn set_written_state(&self, state: Arc<State<T>>) {
        self.written_state.store(Some(state))
    }
}
