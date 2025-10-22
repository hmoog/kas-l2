use std::{collections::HashMap, sync::Arc};

pub trait AppRegistry: Send + Sync {
    fn load_app_elf(&self, app_id: u64) -> Option<Arc<Vec<u8>>>;
}

#[derive(Default)]
pub struct InMemoryAppRegistry {
    map: HashMap<u64, Arc<Vec<u8>>>,
}

impl InMemoryAppRegistry {
    pub fn new() -> Self {
        Self::default()
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
