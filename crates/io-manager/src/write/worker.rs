use std::{
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    thread::JoinHandle,
};

use crossbeam_utils::{
    CachePadded,
    sync::{Parker, Unparker},
};
use kas_l2_io_core::KVStore;

use crate::{cmd_queue::CmdQueue, write};

pub struct Worker<S: KVStore, C: write::Cmd<<S as KVStore>::Namespace>> {
    store: Arc<S>,
    queue: CmdQueue<C>,
    parked: Arc<CachePadded<AtomicBool>>,
    parker: Parker,
    is_shutdown: Arc<AtomicBool>,
}

impl<Store: KVStore, WriteCmd: write::Cmd<<Store as KVStore>::Namespace>> Worker<Store, WriteCmd> {
    pub fn new(
        queue: CmdQueue<WriteCmd>,
        parked: Arc<CachePadded<AtomicBool>>,
        store: Arc<Store>,
        is_shutdown: Arc<AtomicBool>,
    ) -> Self {
        Self {
            queue,
            parked,
            store,
            parker: Parker::new(),
            is_shutdown,
        }
    }

    pub fn unparker(&self) -> Unparker {
        self.parker.unparker().clone()
    }

    pub fn start(self) -> JoinHandle<()> {
        thread::spawn(move || self.run())
    }

    fn run(self) {
        while !self.is_shutdown.load(Ordering::Acquire) {
            match self.queue.pop() {
                (Some(cmd), _) => {
                    cmd.exec(self.store.deref());
                }
                _ => self.park(),
            }
        }
    }

    fn park(&self) {
        self.parked.store(true, Ordering::Relaxed);
        while let (Some(cmd), _) = self.queue.pop() {
            cmd.exec(self.store.deref());
        }
        self.parker.park();
    }
}
