pub trait TransactionProcessor<T>: Fn(&T) + Clone + Send + Sync + 'static {}
impl<T, F: Fn(&T) + Clone + Send + Sync + 'static> TransactionProcessor<T> for F {}
