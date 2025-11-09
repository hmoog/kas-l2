pub trait Task: Send + Sync + 'static {
    fn execute(&self);
}
