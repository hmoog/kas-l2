use std::hash::Hash;

pub trait PubkeyType: Send + Clone + Eq + Hash + 'static {}
impl<T: Send + Clone + Eq + Hash + 'static> PubkeyType for T {}
