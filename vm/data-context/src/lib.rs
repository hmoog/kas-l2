use kas_l2_vm_address::Address;
use kas_l2_vm_authenticated_data::AuthenticatedData;
use kas_l2_vm_error::VmResult;

pub trait DataContext {
    fn borrow(&mut self, address: Address) -> VmResult<&AuthenticatedData>;

    fn borrow_mut(&mut self, address: Address) -> VmResult<&mut AuthenticatedData>;
}
