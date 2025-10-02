pub trait VecExt {
    /// Consuming map -> Vec
    fn into_vec<U, F>(self, f: F) -> Vec<U>
    where
        Self: Sized + IntoIterator,
        F: FnMut(Self::Item) -> U,
    {
        self.into_iter().map(f).collect()
    }

    /// Non-consuming map over `&self`, allowing `U` to borrow from `self`.
    fn as_vec<'s, U, F>(&'s self, f: F) -> Vec<U>
    where
        &'s Self: IntoIterator,
        F: FnMut(<&'s Self as IntoIterator>::Item) -> U,
    {
        self.into_iter().map(f).collect()
    }
}

impl<T> VecExt for T {}
