use std::sync::{Arc, Weak};

use crate::{guard_consumer::GuardConsumer, guard::Guard};

#[must_use = "a GuardsSetup wires guards when dropped - don't ignore it"]
pub struct GuardsSetup<C: GuardConsumer> {
    pub guards: Arc<Vec<Arc<Guard<C>>>>,
    _activator: Activator<C>,
}

impl<C: GuardConsumer> GuardsSetup<C> {
    pub fn new(
        guards: Vec<Arc<Guard<C>>>,
        prev_guards: Vec<Option<Arc<Guard<C>>>>,
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

    pub fn setup(&self, consumer: Weak<C>) -> Arc<Vec<Arc<Guard<C>>>> {
        for guard in self.guards.iter() {
            guard.consumer.store(consumer.clone());
        }
        self.guards.clone()
    }
}

struct Activator<C: GuardConsumer> {
    guards: Arc<Vec<Arc<Guard<C>>>>,
    prev_guards: Vec<Option<Arc<Guard<C>>>>,
}

impl<C: GuardConsumer> Drop for Activator<C> {
    fn drop(&mut self) {
        for (prev_guard, guard) in self.prev_guards.drain(..).zip(self.guards.iter()) {
            match prev_guard {
                None => guard.ready(),
                Some(prev_guard) => prev_guard.extend(guard),
            }
        }
    }
}
