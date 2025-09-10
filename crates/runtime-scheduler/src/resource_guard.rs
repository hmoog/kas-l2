use std::sync::Arc;

use crate::{
    atomic::{AtomicEnum, AtomicWeak},
    scheduled_element::ScheduledElement,
    types::{AccessType, Element, GuardStatus},
};

pub struct ResourceGuard<E: Element> {
    pub status: AtomicEnum<GuardStatus>,
    pub access_type: AtomicEnum<AccessType>,
    pub successor: AtomicWeak<ResourceGuard<E>>,
    pub owner: AtomicWeak<ScheduledElement<E>>,
    pub owner_index: usize,
}

impl<E: Element> ResourceGuard<E> {
    pub fn new(access_type: AccessType, element_index: usize) -> Self {
        Self {
            access_type: AtomicEnum::new(access_type),
            status: AtomicEnum::new(GuardStatus::Waiting),
            successor: AtomicWeak::new(),
            owner: AtomicWeak::new(),
            owner_index: element_index,
        }
    }

    pub fn extend(&self, successor: &Arc<ResourceGuard<E>>) {
        self.successor.store(Arc::downgrade(successor));

        match self.status.load() {
            GuardStatus::Ready if self.access_type.load() == AccessType::ReadAccess => {
                if successor.access_type.load() == AccessType::ReadAccess {
                    successor.ready();
                }
            }
            GuardStatus::Done => {
                successor.ready();
            }
            _ => {} // do nothing, the successor will be notified when anything changes
        }
    }

    pub fn ready(&self) {
        if let Ok(_) = self
            .status
            .compare_exchange(GuardStatus::Waiting, GuardStatus::Ready)
        {
            if let Some(owner) = self.owner.load().upgrade() {
                owner.notify_ready();
            }

            if self.access_type.load() == AccessType::ReadAccess {
                if let Some(successor) = self.successor.load().upgrade() {
                    if successor.access_type.load() == AccessType::ReadAccess {
                        successor.ready();
                    }
                }
            }
        }
    }

    pub fn done(&self) {
        if let Ok(_) = self
            .status
            .compare_exchange(GuardStatus::Ready, GuardStatus::Done)
        {
            if let Some(successor) = self.successor.load().upgrade() {
                successor.ready();
            }
        }
    }
}
