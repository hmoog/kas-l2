pub trait Task: Send + Sync + 'static {
    type ResourceID: kas_l2_resource::ResourceID;

    fn read_locks(&self) -> &[Self::ResourceID];

    fn write_locks(&self) -> &[Self::ResourceID];
}
