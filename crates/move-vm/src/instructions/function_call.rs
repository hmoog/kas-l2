use move_binary_format::errors::VMError;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::ModuleId;
use move_core_types::runtime_value::MoveValue;
use move_vm_runtime::session::SerializedReturnValues;
use move_vm_types::gas::UnmeteredGasMeter;
use move_vm_types::loaded_data::runtime_types::Type;
use crate::ModuleResolver;
use crate::transaction_context::TransactionContext;

pub struct FunctionCall {
    pub module_ref: usize,
    pub function_name: Identifier,
    pub ty_args: Vec<Type>,
    pub args: Vec<FunctionCallArg>,
}

impl FunctionCall {
    pub fn execute(&self, mut ctx: TransactionContext) -> Result<(), VMError> {
        let args = self.args
            .iter()
            .map(|arg| match arg {
                FunctionCallArg::DataRef(index) => ctx.data()[*index].clone(),
                FunctionCallArg::MoveValue(value) => bcs::to_bytes(&value).unwrap(),
            })
            .collect();

        let res = self.call_method(&self.function_name, self.ty_args.clone(), args, ctx)?;
        res.mutable_reference_outputs;

        Ok(())
    }

    fn call_method(
        &self,
        function_name: &Identifier,
        ty_args: Vec<Type>,
        args: Vec<Vec<u8>>,
        mut ctx: TransactionContext,
    ) -> Result<SerializedReturnValues, VMError> {
        let result = ctx.session.execute_entry_function(
            &ctx.modules().id(self.module_ref),
            function_name,
            ty_args.clone(),
            args,
            &mut UnmeteredGasMeter,
        )?;

        ctx.session.finish().0.expect("session should be finished");

        Ok(result)
    }
}


pub enum FunctionCallArg {
    DataRef(usize),
    MoveValue(MoveValue),
}