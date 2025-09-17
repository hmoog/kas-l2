use std::sync::{Arc, Weak};

use crate::{consumer::Consumer, resource_guard::ResourceGuard};

#[must_use = "a GuardsSetup wires guards when dropped - don't ignore it"]
pub struct GuardsSetup<C: Consumer> {
    pub guards: Arc<Vec<Arc<ResourceGuard<C>>>>,
    _activator: Activator<C>,
}

impl<C: Consumer> GuardsSetup<C> {
    pub fn new(
        guards: Vec<Arc<ResourceGuard<C>>>,
        prev_guards: Vec<Option<Arc<ResourceGuard<C>>>>,
    ) -> Self {
        let guards = Arc::new(guards);
        Self {
            guards: guards.clone(),
            _activator: Activator {
                guards,
                prev_guards,
            },
        }
    }

    pub fn setup(&self, consumer: Weak<C>) -> Arc<Vec<Arc<ResourceGuard<C>>>> {
        for guard in self.guards.iter() {
            guard.notifier.store(consumer.clone());
        }
        self.guards.clone()
    }
}

struct Activator<C: Consumer> {
    guards: Arc<Vec<Arc<ResourceGuard<C>>>>,
    prev_guards: Vec<Option<Arc<ResourceGuard<C>>>>,
}

impl<C: Consumer> Drop for Activator<C> {
    fn drop(&mut self) {
        for (prev_guard, guard) in self.prev_guards.drain(..).zip(self.guards.iter()) {
            match prev_guard {
                None => guard.ready(),
                Some(prev_guard) => prev_guard.extend(guard),
            }
        }
    }
}
