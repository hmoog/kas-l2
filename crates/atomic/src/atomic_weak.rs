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
