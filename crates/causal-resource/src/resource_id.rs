use std::hash::Hash;

pub trait ResourceID: Eq + Hash + Clone + Send + Sync + 'static {}
impl<T: Eq + Hash + Clone + Send + Sync + 'static> ResourceID for T {}
