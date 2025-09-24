use std::sync::{
    Arc,
    atomic::{AtomicPtr, Ordering},
};

pub struct AtomicArc<T> {
    ptr: AtomicPtr<T>,
}

impl<T> AtomicArc<T> {
    pub fn new(value: Arc<T>) -> Self {
        Self {
            ptr: AtomicPtr::new(Arc::into_raw(value) as *mut T),
        }
    }

    pub fn store(&self, value: Arc<T>) {
        let new = Arc::into_raw(value) as *mut T;
        let old = self.ptr.swap(new, Ordering::Release);
        if !old.is_null() {
            unsafe { Arc::from_raw(old) }; // drop the old Arc
        }
    }

    pub fn load(&self) -> Arc<T> {
        let raw = self.ptr.load(Ordering::Acquire);
        if raw.is_null() {
            panic!("AtomicArc is uninitialized");
        } else {
            unsafe {
                let a = Arc::from_raw(raw);
                let cloned = a.clone();
                std::mem::forget(a); // avoid decrementing the strong count
                cloned
            }
        }
    }
}

impl<T> Drop for AtomicArc<T> {
    fn drop(&mut self) {
        let raw = *self.ptr.get_mut();
        if !raw.is_null() {
            unsafe { Arc::from_raw(raw) }; // drop the last Arc
        }
    }
}
