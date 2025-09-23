use std::sync::{Arc, Weak};

use kas_l2_atomic::{AtomicEnum, AtomicOptionArc, AtomicWeak};
use kas_l2_core::{AccessType, ResourceState, Transaction};

use crate::resource::{access_status::AccessStatus, resource_consumer::ResourceConsumer};

pub struct ResourceProvider<T: Transaction, C: ResourceConsumer<T>> {
    status: AtomicEnum<AccessStatus>,
    pub(crate) received_value: AtomicOptionArc<ResourceState<T>>,
    produced_value: AtomicOptionArc<ResourceState<T>>,
    pub access_type: AccessType,
    pub consumer: (AtomicWeak<C>, C::ResourceID),
    pub prev: AtomicOptionArc<Self>,
    pub next: AtomicWeak<Self>,
}

impl<T: Transaction, C: ResourceConsumer<T>> ResourceProvider<T, C> {
    pub fn new(
        prev: Option<Arc<Self>>,
        consumer: Weak<C>,
        consumer_guard_id: C::ResourceID,
        access_type: AccessType,
    ) -> Self {
        Self {
            received_value: AtomicOptionArc::empty(),
            produced_value: AtomicOptionArc::empty(),
            access_type,
            status: AtomicEnum::new(AccessStatus::Waiting),
            consumer: (AtomicWeak::new(consumer), consumer_guard_id),
            prev: AtomicOptionArc::new(prev),
            next: AtomicWeak::default(),
        }
    }

    pub fn extend(&self, successor: &Arc<Self>) {
        self.next.store(Arc::downgrade(successor));

        self.produced_value.load().map(|state| successor.receive_state(state));
    }

    pub fn receive_state(self: &Arc<Self>, state: Arc<ResourceState<T>>) {
        if self.received_value.publish(state.clone()) {
            if let Some(owner) = self.consumer.0.load().upgrade() {
                owner.notify(self.clone());
            }

            if self.access_type == AccessType::Read {
                self.produce_value(state);
            }
        }
    }

    pub fn produce_value(&self, state: Arc<ResourceState<T>>) {
        if self.produced_value.publish(state.clone()) {
            self.next.load().upgrade().map(|next| next.receive_state(state));
        }
    }
}
