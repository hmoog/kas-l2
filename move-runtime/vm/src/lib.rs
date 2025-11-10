pub use instruction::{Instruction, MethodCallArg};
pub(crate) use modules::Modules;
pub use object_access::ObjectAccess;
pub use object_id::ObjectId;
pub use transaction::Transaction;
pub use vm::Vm;

mod execution_context;
mod instruction;
mod modules;
mod object_access;
mod object_id;
mod ownership;
mod transaction;
mod vm;
