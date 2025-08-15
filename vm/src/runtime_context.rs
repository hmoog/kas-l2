use std::sync::Arc;

use solana_sbpf::vm::ContextObject;

use crate::app_registry::AppRegistry;
use crate::runtime_state::RuntimeState;

pub struct RuntimeContext {
    pub runtime_state: RuntimeState,
    pub app_registry: Arc<dyn AppRegistry>,
    remaining_gas: u64,
}

impl RuntimeContext {
    pub fn new(app_registry: Arc<dyn AppRegistry>, runtime_state: RuntimeState, gas_limit: u64) -> Self {
        Self {
            app_registry,
            runtime_state,
            remaining_gas: gas_limit,
        }
    }
}

impl ContextObject for RuntimeContext {
    fn trace(&mut self, _state: [u64; 12]) {
        /* optional tracing */
    }

    fn consume(&mut self, amount: u64) {
        self.remaining_gas = self.remaining_gas.saturating_sub(amount);
    }

    fn get_remaining(&self) -> u64 {
        self.remaining_gas
    }
}
