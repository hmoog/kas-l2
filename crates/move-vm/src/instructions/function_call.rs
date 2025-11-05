use move_binary_format::errors::VMError;
use move_core_types::identifier::Identifier;
use move_core_types::runtime_value::MoveValue;
use move_vm_types::gas::UnmeteredGasMeter;
use move_vm_types::loaded_data::runtime_types::Type;
use crate::execution_context::ExecutionContext;

pub struct FunctionCall {
    pub module_ref: usize,
    pub function_name: Identifier,
    pub ty_args: Vec<Type>,
    pub args: Vec<FunctionCallArg>,
}

impl FunctionCall {
    pub fn execute(&self, mut ctx: ExecutionContext) -> Result<(), VMError> {
        let execution_results = ctx.session.execute_entry_function(
            &ctx.modules().id(self.module_ref),
            &self.function_name,
            self.ty_args.clone(),
            ctx.serialized_args(&self.args),
            &mut UnmeteredGasMeter,
        )?;
        ctx.ingest_execution_results(execution_results);
        ctx.finalize()
    }
}


pub enum FunctionCallArg {
    DataRef(usize),
    MoveValue(MoveValue),
}