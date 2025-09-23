use std::hash::Hash;

pub trait ResourceID: Default + Eq + Hash + Clone + Send + Sync + 'static {}
impl<T: Default + Eq + Hash + Clone + Send + Sync + 'static> ResourceID for T {}
