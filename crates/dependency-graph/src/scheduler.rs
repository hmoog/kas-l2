use crate::access_type::AccessType;
use crate::element::Element;
use crate::lock_request::LockRequest;
use crate::scheduled_element::ScheduledElement;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Scheduler<E: Element> {
    scheduled_elements: Vec<Arc<ScheduledElement<E>>>,
    lock_requests: HashMap<E::ResourceID, Arc<LockRequest<E>>>,
}

impl<E: Element> Scheduler<E> {
    pub fn new() -> Self {
        Self {
            scheduled_elements: Vec::new(),
            lock_requests: HashMap::new(),
        }
    }

    pub fn schedule(&mut self, elements: Vec<E>) {
        for (i, element) in elements.into_iter().enumerate() {
            let scheduled_element = self.schedule_element(i, element);
            self.scheduled_elements.push(scheduled_element);
        }
    }

    pub fn schedule_element(&mut self, index: usize, element: E) -> Arc<ScheduledElement<E>> {
        let mut old_requests = Vec::new();
        let mut new_requests = Vec::new();

        let mut collect = |locks: &[E::ResourceID], access: AccessType| {
            for res in locks {
                let (old_request, new_request) = match self.lock_requests.entry(res.clone()) {
                    Entry::Occupied(entry) if entry.get().element_index == index => {
                        continue; // skip duplicate lock for the same element
                    }
                    Entry::Occupied(mut entry) => {
                        let new_request = Arc::new(LockRequest::new(access, index));
                        let old_request = Some(entry.insert(new_request.clone()));
                        (old_request, new_request)
                    }
                    Entry::Vacant(entry) => {
                        let new_request = Arc::new(LockRequest::new(access, index));
                        entry.insert(new_request.clone());
                        (None, new_request)
                    }
                };

                old_requests.push(old_request);
                new_requests.push(new_request);
            }
        };

        collect(element.write_locks(), AccessType::Write);
        collect(element.read_locks(), AccessType::Read);

        ScheduledElement::new(element, index, new_requests, old_requests)
    }
}
