use std::hash::Hash;

pub trait Task: Send + Sync + 'static {
    type ResourceID: Eq + Hash + Clone + Sync + Send + 'static;

    fn read_locks(&self) -> &[Self::ResourceID];

    fn write_locks(&self) -> &[Self::ResourceID];
}
