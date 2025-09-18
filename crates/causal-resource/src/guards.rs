use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use kas_l2_atomic::AtomicWeak;
use crate::{GuardConsumer, Guard};

pub struct Guards<C: GuardConsumer<GuardID=usize>> {
    guards: Vec<AtomicWeak<Guard<C>>>,
    pending_guards: AtomicU64,
    consumer: AtomicWeak<C>,
}

impl<C: GuardConsumer<GuardID=usize>> Guards<C> {
    pub fn new(guards: Vec<Arc<Guard<C>>>) -> Arc<Self> {
        Arc::new(Self {
            pending_guards: AtomicU64::new(guards.len() as u64),
            consumer: AtomicWeak::default(),
            guards: Vec::new(),
        })
    }

    pub fn notify(self: &Arc<Self>, consumer: Arc<C>) {
        self.consumer.store(Arc::downgrade(&consumer));

        // TODO: if already done, notify immediately
    }
}

impl<C: GuardConsumer<GuardID=usize>> GuardConsumer for Guards<C> {
    type GuardID = usize;

    fn notify(self: &Arc<Self>, key: &C::GuardID) {
        if self.pending_guards.fetch_sub(1, Ordering::AcqRel) == 1 {
            if let Some(consumer) = self.consumer.load().upgrade() {
                consumer.notify(key);
            }
        } else {
            eprintln!("Guards::notify: consumer is gone");
        }
    }
}
