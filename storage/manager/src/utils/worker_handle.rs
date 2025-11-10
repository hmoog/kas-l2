use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
};

use crossbeam_utils::{CachePadded, atomic::AtomicCell, sync::Unparker};

pub struct WorkerHandle {
    unparker: Unparker,
    is_parked: Arc<CachePadded<AtomicBool>>,
    join_handle: AtomicCell<Option<JoinHandle<()>>>,
}

impl WorkerHandle {
    pub(crate) fn new(
        unparker: Unparker,
        is_parked: Arc<CachePadded<AtomicBool>>,
        join_handle: JoinHandle<()>,
    ) -> Self {
        Self { unparker, is_parked, join_handle: AtomicCell::new(Some(join_handle)) }
    }

    #[inline]
    pub(crate) fn is_parked(&self) -> bool {
        self.is_parked.load(Ordering::Relaxed)
    }

    #[inline]
    pub(crate) fn wake(&self) {
        self.unparker.unpark();
    }

    pub(crate) fn take_join(&self) -> Option<JoinHandle<()>> {
        self.join_handle.take()
    }
}
