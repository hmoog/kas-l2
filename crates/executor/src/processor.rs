pub trait Processor<T>: Fn(&T) + Clone + Send + Sync + 'static {}
impl<T, F: Fn(&T) + Clone + Send + Sync + 'static> Processor<T> for F {}
