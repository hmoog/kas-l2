use crate::io::state_space::StateSpace;

pub trait WriteBatch<'a> {
    fn put(&mut self, ns: StateSpace, key: &[u8], value: &[u8]);
    fn delete(&mut self, ns: StateSpace, key: &[u8]);
}
