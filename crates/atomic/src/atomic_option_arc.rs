use std::{
    ptr,
    sync::{
        Arc,
        atomic::{AtomicPtr, Ordering},
    },
};

pub struct AtomicOptionArc<T> {
    ptr: AtomicPtr<T>,
}

impl<T> AtomicOptionArc<T> {
    pub fn new(value: Option<Arc<T>>) -> Self {
        Self {
            ptr: match value {
                Some(value) => AtomicPtr::new(Arc::into_raw(value) as *mut T),
                None => AtomicPtr::new(ptr::null_mut()),
            },
        }
    }

    pub fn empty() -> Self {
        Self {
            ptr: AtomicPtr::new(ptr::null_mut()),
        }
    }

    pub fn store(&self, value: Option<Arc<T>>) {
        let new = value
            .map(|arc| Arc::into_raw(arc) as *mut T)
            .unwrap_or(ptr::null_mut());
        let old = self.ptr.swap(new, Ordering::Release);
        if !old.is_null() {
            unsafe { Arc::from_raw(old) }; // drop the old Arc
        }
    }

    pub fn load(&self) -> Option<Arc<T>> {
        let raw = self.ptr.load(Ordering::Acquire);
        if raw.is_null() {
            None
        } else {
            unsafe {
                let a = Arc::from_raw(raw);
                let cloned = a.clone();
                std::mem::forget(a); // avoid decrementing the strong count
                Some(cloned)
            }
        }
    }

    pub fn take(&self) -> Option<Arc<T>> {
        let raw = self.ptr.swap(ptr::null_mut(), Ordering::AcqRel);
        if raw.is_null() {
            None
        } else {
            unsafe { Some(Arc::from_raw(raw)) }
        }
    }
}

impl<T> Drop for AtomicOptionArc<T> {
    fn drop(&mut self) {
        let raw = *self.ptr.get_mut();
        if !raw.is_null() {
            unsafe { Arc::from_raw(raw) }; // drop the last Arc
        }
    }
}
