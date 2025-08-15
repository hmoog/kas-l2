use std::sync::Arc;

use solana_sbpf::vm::ContextObject;

use crate::app_registry::AppRegistry;
use crate::runtime_state::RuntimeState;

pub struct ExecutionContext {
    remaining_gas: u64,
    pub runtime_state: RuntimeState,
    pub app_registry: Arc<dyn AppRegistry>,
}

impl ExecutionContext {
    pub fn new(registry: Arc<dyn AppRegistry>, runtime: RuntimeState, gas_limit: u64) -> Self {
        Self {
            remaining_gas: gas_limit,
            app_registry: registry,
            runtime_state: runtime,
        }
    }
}

impl ContextObject for ExecutionContext {
    fn trace(&mut self, _state: [u64; 12]) { /* optional tracing */
    }

    fn consume(&mut self, amount: u64) {
        self.remaining_gas = self.remaining_gas.saturating_sub(amount);
    }

    fn get_remaining(&self) -> u64 {
        self.remaining_gas
    }
}
