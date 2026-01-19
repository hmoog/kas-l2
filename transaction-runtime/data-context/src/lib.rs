use vprogs_transaction_runtime_address::Address;
use vprogs_transaction_runtime_authenticated_data::AuthenticatedData;
use vprogs_transaction_runtime_error::VmResult;

pub trait DataContext {
    fn borrow(&mut self, address: Address) -> VmResult<&AuthenticatedData>;

    fn borrow_mut(&mut self, address: Address) -> VmResult<&mut AuthenticatedData>;
}
