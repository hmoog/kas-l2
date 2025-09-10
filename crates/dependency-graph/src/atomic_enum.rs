use std::convert::TryFrom;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU8, Ordering};

pub struct AtomicEnum<T> {
    inner: AtomicU8,
    _marker: PhantomData<T>,
}

impl<T> AtomicEnum<T>
where
    T: Into<u8> + TryFrom<u8>,
{
    pub fn new(value: T) -> Self {
        Self {
            inner: AtomicU8::new(value.into()),
            _marker: PhantomData,
        }
    }

    pub fn load(&self) -> T {
        T::try_from(self.inner.load(Ordering::Acquire))
            .ok()
            .unwrap()
    }

    pub fn store(&self, value: T) {
        self.inner.store(value.into(), Ordering::Release)
    }

    pub fn swap(&self, value: T) -> T {
        T::try_from(self.inner.swap(value.into(), Ordering::AcqRel))
            .ok()
            .unwrap()
    }

    pub fn compare_exchange(&self, current: T, new: T) -> Result<T, T> {
        self.inner
            .compare_exchange(
                current.into(),
                new.into(),
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|v| T::try_from(v).ok().unwrap())
            .map_err(|v| T::try_from(v).ok().unwrap())
    }

    pub fn compare_exchange_weak(&self, current: T, new: T) -> Result<T, T> {
        self.inner
            .compare_exchange_weak(
                current.into(),
                new.into(),
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|v| T::try_from(v).ok().unwrap())
            .map_err(|v| T::try_from(v).ok().unwrap())
    }
}
