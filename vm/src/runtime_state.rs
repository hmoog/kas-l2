use std::sync::Arc;

#[derive(Clone)]
pub struct RuntimeState {
    pub state_root: blake3::Hash,
}

impl RuntimeState {
    pub fn new(initial_root: blake3::Hash) -> Arc<Self> {
        Arc::new(Self {
            state_root: initial_root,
        })
    }
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            state_root: blake3::Hash::from([0u8; 32]),
        }
    }
}