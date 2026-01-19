use kas_l2_runtime_manager::VmInterface;
use kas_l2_runtime_state::StateSpace;
use kas_l2_storage_types::Store;
use kas_l2_transaction_runtime_address::Address;
use kas_l2_transaction_runtime_authenticated_data::AuthenticatedData;
use kas_l2_transaction_runtime_data_context::DataContext;
use kas_l2_transaction_runtime_error::{VmError, VmResult};
use kas_l2_transaction_runtime_lock::Lock;
use kas_l2_transaction_runtime_object_id::ObjectId;

use crate::TransactionRuntime;

impl<'a, 'b, S, V> DataContext for TransactionRuntime<'a, 'b, S, V>
where
    S: Store<StateSpace = StateSpace>,
    V: VmInterface<ResourceId = ObjectId, Ownership = Lock>,
{
    fn borrow(&mut self, address: Address) -> VmResult<&AuthenticatedData> {
        self.loaded_data.get(&address).ok_or(VmError::DataNotFound(address))
    }

    fn borrow_mut(&mut self, address: Address) -> VmResult<&mut AuthenticatedData> {
        self.loaded_data.get_mut(&address).ok_or(VmError::DataNotFound(address)).and_then(|data| {
            match data.mut_cap().is_some() {
                true => Ok(data),
                false => Err(VmError::MissingMutCapability(address)),
            }
        })
    }
}
