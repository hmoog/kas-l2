pub trait Transaction: Send + Sync + 'static {
    type ResourceID: crate::ResourceID;

    fn read_locks(&self) -> &[Self::ResourceID];

    fn write_locks(&self) -> &[Self::ResourceID];
}
