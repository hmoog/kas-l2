pub trait Transaction: Send + Sync + 'static {
    type ResourceID: crate::ResourceID;
    type AccessMetadata: crate::AccessMetadata<Self::ResourceID>;

    fn accessed_resources(&self) -> &[Self::AccessMetadata];
}
