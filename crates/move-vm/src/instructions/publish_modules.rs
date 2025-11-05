use move_binary_format::errors::VMError;
use move_core_types::account_address::AccountAddress;
use move_core_types::effects::Op;
use move_core_types::language_storage::ModuleId;
use move_vm_types::gas::UnmeteredGasMeter;
use crate::instructions::function_call::FunctionCallArg;
use crate::ModuleResolver;
use crate::transaction_context::TransactionContext;

pub struct PublishModules {
    pub modules: Vec<Vec<u8>>,
    pub sender: AccountAddress,
}

impl PublishModules {
    pub fn execute(&self, mut ctx: TransactionContext) -> Result<(), VMError> {
        for (_module_id, _op) in self.publish_modules(self.modules.clone(), self.sender.clone(), ctx)? {
            // let Some(resource) = resources_by_id.get_mut(&module_id) else {
            //     panic!("no resource found for published module {:?}", module_id);
            // };
            //
            // match op {
            //     Op::New(data) | Op::Modify(data) => {
            //         resource.state_mut().data = data;
            //     }
            //     Op::Delete => panic!("published module cannot be deleted"),
            // }
        }

        Ok(())
    }

    fn publish_modules(
        &self,
        modules_bytes: Vec<Vec<u8>>,
        sender: AccountAddress,
        mut ctx: TransactionContext,
    ) -> Result<Vec<(ModuleId, Op<Vec<u8>>)>, VMError> {
        ctx.session.publish_module_bundle(modules_bytes, sender, &mut UnmeteredGasMeter)?;
        Ok(ctx.session.finish().0?.into_modules().collect())
    }
}