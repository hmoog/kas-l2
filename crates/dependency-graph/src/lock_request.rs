use crate::access_type::AccessType;
use crate::atomic_enum::AtomicEnum;
use crate::atomic_weak::AtomicWeak;
use crate::element::Element;
use crate::lock_status::LockStatus;
use crate::scheduled_element::ScheduledElement;
use std::sync::Arc;

pub struct LockRequest<E: Element> {
    pub lock_type: AtomicEnum<AccessType>,
    pub lock_status: AtomicEnum<LockStatus>,
    pub notify_successor: AtomicWeak<LockRequest<E>>,
    pub owner: AtomicWeak<ScheduledElement<E>>,
    pub element_index: usize,
}

impl<E: Element> LockRequest<E> {
    pub fn new(lock_type: AccessType, element_index: usize) -> Self {
        Self {
            lock_type: AtomicEnum::new(lock_type),
            lock_status: AtomicEnum::new(LockStatus::Requested),
            notify_successor: AtomicWeak::new(),
            owner: AtomicWeak::new(),
            element_index,
        }
    }

    pub fn notify(&self, successor: &Arc<LockRequest<E>>) {
        self.notify_successor.store(Arc::downgrade(successor));

        match self.lock_status.load() {
            LockStatus::Acquired if self.lock_type.load() == AccessType::Read => {
                if successor.lock_type.load() == AccessType::Read {
                    successor.acquire();
                }
            }
            LockStatus::Released => {
                successor.acquire();
            }
            _ => {
                // do nothing, the successor will be notified when this lock is released
            }
        }
    }

    pub fn acquire(&self) {
        if self
            .lock_status
            .compare_exchange(LockStatus::Requested, LockStatus::Acquired)
            .is_ok()
        {
            if let Some(owner) = self.owner.load().upgrade() {
                owner.acquire_lock();
            }

            if self.lock_type.load() == AccessType::Read {
                if let Some(successor) = self.notify_successor.load().upgrade() {
                    if successor.lock_type.load() == AccessType::Read {
                        successor.acquire();
                    }
                }
            }
        }
    }

    pub fn release(&self) {
        if self
            .lock_status
            .compare_exchange(LockStatus::Acquired, LockStatus::Released)
            .is_ok()
        {
            if let Some(successor) = self.notify_successor.load().upgrade() {
                successor.acquire();
            }
        }
    }
}
