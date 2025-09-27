use std::sync::{Arc, Weak};

use crate::{
    resources::{Consumer, Resources, resource::Resource},
    transactions::Transaction,
};

pub(crate) struct ResourceManager<T: Transaction, C: Consumer> {
    last_resource: Option<Arc<Resource<T, C>>>,
}

impl<T: Transaction, C: Consumer> Default for ResourceManager<T, C> {
    fn default() -> Self {
        Self {
            last_resource: None,
        }
    }
}

impl<T: Transaction, C: Consumer> ResourceManager<T, C> {
    pub(crate) fn provide_resource(
        &mut self,
        metadata: T::AccessMetadata,
        consumer: Weak<Resources<T, C>>,
    ) -> Arc<Resource<T, C>> {
        let access = Resource::new(consumer, self.last_resource.take(), metadata);
        self.last_resource = Some(access.clone());
        access
    }

    pub(crate) fn is_duplicate_access(&self, resources: &Weak<Resources<T, C>>) -> bool {
        match self.last_resource.as_ref() {
            Some(last_resource) => last_resource.belongs_to(resources),
            None => false,
        }
    }
}
