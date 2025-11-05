use move_binary_format::errors::{VMError};
use move_core_types::account_address::AccountAddress;
use move_vm_types::gas::UnmeteredGasMeter;
use crate::execution_context::ExecutionContext;

pub struct PublishModules {
    pub modules: Vec<Vec<u8>>,
    pub sender: AccountAddress,
}

impl PublishModules {
    pub fn execute(&self, mut ctx: ExecutionContext) -> Result<(), VMError> {
        ctx.session.publish_module_bundle(self.modules.clone(), self.sender, &mut UnmeteredGasMeter)?;
        ctx.finalize()
    }
}