use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::Notify;

/// A one-shot async latch.
///
/// Starts "closed". Once opened, it stays open forever.
/// All current and future waiters will observe it as open.
pub struct AtomicAsyncLatch {
    ready: AtomicBool,
    notify: Notify,
}

impl AtomicAsyncLatch {
    /// Create a new latch in the "closed" state.
    pub fn new() -> Self {
        Self {
            ready: AtomicBool::new(false),
            notify: Notify::new(),
        }
    }

    /// Open the latch (transition from false → true).
    /// Wakes all current waiters. Idempotent.
    pub fn open(&self) -> bool {
        if !self.ready.swap(true, Ordering::Release) {
            self.notify.notify_waiters();
            true
        } else {
            false
        }
    }

    /// Returns whether the latch is already open.
    pub fn is_open(&self) -> bool {
        self.ready.load(Ordering::Acquire)
    }

    /// Wait until the latch is open.
    /// Returns immediately if it's already open.
    pub async fn wait(&self) {
        let notified = self.notify.notified();

        if self.ready.load(Ordering::Acquire) {
            return;
        }

        notified.await;
    }
}

impl Default for AtomicAsyncLatch {
    fn default() -> Self {
        Self::new()
    }
}
