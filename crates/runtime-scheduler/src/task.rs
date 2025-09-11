use std::hash::Hash;

pub trait Task {
    type ResourceID: Eq + Hash + Clone;

    fn read_locks(&self) -> &[Self::ResourceID];

    fn write_locks(&self) -> &[Self::ResourceID];
}
