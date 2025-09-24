use std::{
    ops::Deref,
    sync::{Arc, Weak},
};

use kas_l2_atomic::{AtomicOptionArc, AtomicWeak};

use crate::{
    AccessMetadata, AccessType, ResourceState, ResourcesConsumer, ResourcesProvider, Transaction,
};

pub struct ResourceProvider<T: Transaction, R: ResourcesConsumer> {
    access_metadata: T::AccessMetadata,
    consumer: (Weak<ResourcesProvider<T, R>>, usize),
    prev: AtomicOptionArc<Self>,
    next: AtomicWeak<Self>,
    read_value: AtomicOptionArc<ResourceState<T>>,
    written_value: AtomicOptionArc<ResourceState<T>>,
}

impl<T: Transaction, R: ResourcesConsumer> ResourceProvider<T, R> {
    pub fn new(
        access_metadata: T::AccessMetadata,
        consumer: (Weak<ResourcesProvider<T, R>>, usize),
        prev: Option<Arc<Self>>,
    ) -> Self {
        Self {
            access_metadata,
            consumer,
            prev: AtomicOptionArc::new(prev),
            next: AtomicWeak::default(),
            read_value: AtomicOptionArc::empty(),
            written_value: AtomicOptionArc::empty(),
        }
    }

    pub fn prev(&self) -> Option<Arc<Self>> {
        self.prev.load()
    }

    pub fn has_consumer(&self, consumer: &Weak<ResourcesProvider<T, R>>) -> bool {
        Weak::ptr_eq(&self.consumer.0, consumer)
    }

    pub fn consumer(&self) -> &(Weak<ResourcesProvider<T, R>>, usize) {
        &self.consumer
    }

    pub fn extend(&self, successor: &Arc<Self>) {
        self.next.store(Arc::downgrade(successor));

        if let Some(state) = self.written_value.load() {
            successor.publish_read_value(state);
        }
    }

    pub fn read_value(&self) -> Option<Arc<ResourceState<T>>> {
        self.read_value.load()
    }

    pub fn publish_read_value(self: &Arc<Self>, state: Arc<ResourceState<T>>) {
        if self.read_value.publish(state.clone()) {
            drop(self.prev.take()); // allow the previous provider to be dropped

            if let Some(owner) = self.consumer.0.upgrade() {
                owner.notify(self.clone());
            }

            if self.access_type() == AccessType::Read {
                self.publish_written_value(state);
            }
        }
    }

    pub fn publish_written_value(&self, state: Arc<ResourceState<T>>) {
        if self.written_value.publish(state.clone()) {
            if let Some(next) = self.next.load().upgrade() {
                next.publish_read_value(state)
            }
        }
    }
}

impl<T: Transaction, C: ResourcesConsumer> Deref for ResourceProvider<T, C> {
    type Target = T::AccessMetadata;

    fn deref(&self) -> &Self::Target {
        &self.access_metadata
    }
}
