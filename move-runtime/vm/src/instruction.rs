use move_binary_format::errors::VMResult;
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, runtime_value::MoveValue,
};
use move_vm_types::{gas::UnmeteredGasMeter, loaded_data::runtime_types::Type};

use crate::execution_context::ExecutionContext;

pub enum Instruction {
    PublishModules { modules: Vec<Vec<u8>>, sender: AccountAddress },
    MethodCall { module: usize, fn_name: Identifier, ty_args: Vec<Type>, args: Vec<MethodCallArg> },
}

impl Instruction {
    pub fn execute(&self, mut ctx: ExecutionContext) -> VMResult<()> {
        match self {
            Instruction::PublishModules { modules, sender } => ctx.session.publish_module_bundle(
                modules.clone(),
                *sender,
                &mut UnmeteredGasMeter,
            )?,

            Instruction::MethodCall { module, fn_name, ty_args, args } => {
                let args = ctx.prepare_args(args);
                let execution_results = ctx.session.execute_function_bypass_visibility(
                    &ctx.modules().id(*module),
                    fn_name,
                    ty_args.clone(),
                    args,
                    &mut UnmeteredGasMeter,
                    None,
                )?;
                ctx.ingest_execution_results(execution_results);
            }
        }

        ctx.finalize()
    }
}

pub enum MethodCallArg {
    DataRef(usize),
    MoveValue(MoveValue),
}
