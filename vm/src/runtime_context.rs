use std::sync::Arc;

use solana_sbpf::vm::ContextObject;

use crate::app_registry::AppRegistry;
use crate::runtime_state::RuntimeState;

pub struct RuntimeContext {
    pub runtime_state: RuntimeState,
    pub app_registry: Arc<dyn AppRegistry>,
    pub heap_cursor: u64,
    pub heap_end: u64,
    remaining_gas: u64,
}

impl RuntimeContext {
    pub fn new(
        app_registry: Arc<dyn AppRegistry>,
        runtime_state: RuntimeState,
        gas_limit: u64,
    ) -> Self {
        Self {
            heap_cursor: 0,
            heap_end: 0,
            app_registry,
            runtime_state,
            remaining_gas: gas_limit,
        }
    }
}

impl ContextObject for RuntimeContext {
    fn trace(&mut self, state: [u64; 12]) {
        let r1 = state[1];
        let r2 = state[2];
        let r10 = state[10];
        let pc = state[11]; // program counter
        eprintln!("pc={:#06x}  r1={:#x}  r2={:#x}  r10={:#x}", pc, r1, r2, r10);
    }

    fn consume(&mut self, amount: u64) {
        self.remaining_gas = self.remaining_gas.saturating_sub(amount);
    }

    fn get_remaining(&self) -> u64 {
        self.remaining_gas
    }
}
