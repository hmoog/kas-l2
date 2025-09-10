pub use atomic_enum::AtomicEnum;
pub use atomic_weak::AtomicWeak;

mod atomic_enum {
    use std::{
        marker::PhantomData,
        sync::atomic::{AtomicU8, Ordering},
    };

    pub struct AtomicEnum<T: Into<u8> + TryFrom<u8>> {
        inner: AtomicU8,
        _marker: PhantomData<T>,
    }

    impl<T: Into<u8> + TryFrom<u8>> AtomicEnum<T> {
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
}

mod atomic_weak {
    use std::{
        ptr,
        sync::{
            Weak,
            atomic::{AtomicPtr, Ordering},
        },
    };

    pub struct AtomicWeak<T> {
        ptr: AtomicPtr<T>,
    }

    impl<T> AtomicWeak<T> {
        pub fn new() -> Self {
            Self {
                ptr: AtomicPtr::new(ptr::null_mut()),
            }
        }

        pub fn store(&self, value: Weak<T>) {
            let new = Weak::into_raw(value) as *mut T;
            let old = self.ptr.swap(new, Ordering::Release);
            if !old.is_null() {
                unsafe { Weak::from_raw(old) }; // drop the old weak
            }
        }

        pub fn load(&self) -> Weak<T> {
            let raw = self.ptr.load(Ordering::Acquire);
            if raw.is_null() {
                Weak::new()
            } else {
                unsafe {
                    let w = Weak::from_raw(raw);
                    let cloned = w.clone();
                    std::mem::forget(w);
                    cloned
                }
            }
        }
    }

    impl<T> Drop for AtomicWeak<T> {
        fn drop(&mut self) {
            let raw = *self.ptr.get_mut();
            if !raw.is_null() {
                unsafe { Weak::from_raw(raw) };
            }
        }
    }
}
