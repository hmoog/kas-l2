use std::collections::HashMap;
use std::sync::Arc;

/// How the runtime locates application binaries.
/// In a real node this might hit a DB, local cache, or network.
pub trait AppRegistry: Send + Sync {
    /// Returns ELF bytes for `app_id`, if known.
    fn load_app_elf(&self, app_id: u64) -> Option<Arc<Vec<u8>>>;
}

/// A tiny in-memory registry for demos/tests.
pub struct InMemoryAppRegistry {
    map: HashMap<u64, Arc<Vec<u8>>>,
}

impl InMemoryAppRegistry {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }
    pub fn insert(&mut self, app_id: u64, elf: Vec<u8>) {
        self.map.insert(app_id, Arc::new(elf));
    }
}

impl AppRegistry for InMemoryAppRegistry {
    fn load_app_elf(&self, app_id: u64) -> Option<Arc<Vec<u8>>> {
        self.map.get(&app_id).cloned()
    }
}
