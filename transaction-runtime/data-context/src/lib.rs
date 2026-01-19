use kas_l2_transaction_runtime_address::Address;
use kas_l2_transaction_runtime_authenticated_data::AuthenticatedData;
use kas_l2_transaction_runtime_error::VmResult;

pub trait DataContext {
    fn borrow(&mut self, address: Address) -> VmResult<&AuthenticatedData>;

    fn borrow_mut(&mut self, address: Address) -> VmResult<&mut AuthenticatedData>;
}
