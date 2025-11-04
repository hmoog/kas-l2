use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, runtime_value::MoveValue,
};
use move_vm_types::loaded_data::runtime_types::Type;

pub enum Instruction {
    MethodCall {
        module_ref: usize,
        function_name: Identifier,
        ty_args: Vec<Type>,
        args: Vec<MethodArg>,
    },
    PublishModules {
        modules: Vec<Vec<u8>>,
        sender: AccountAddress,
    },
}

pub enum MethodArg {
    DataRef(usize),
    MoveValue(MoveValue),
}
