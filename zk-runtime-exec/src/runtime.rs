use std::sync::Arc;

/// Extremely small “state” — just a root hash to show state transitions.
#[derive(Clone)]
pub struct RuntimeState {
    pub state_root: blake3::Hash,
}

impl RuntimeState {
    pub fn new(initial_root: blake3::Hash) -> Arc<Self> {
        Arc::new(Self { state_root: initial_root })
    }
}
