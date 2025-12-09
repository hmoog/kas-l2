use kas_l2_runtime_manager::VmInterface;
use kas_l2_runtime_state::StateSpace;
use kas_l2_storage_types::Store;
use kas_l2_vm_crypto::Address;
use kas_l2_vm_lock::Lock;
use kas_l2_vm_object_id::ObjectId;
use kas_l2_vm_transaction_context::TransactionContext;

pub struct ProgramContext<
    'e,
    'a,
    'b,
    S: Store<StateSpace = StateSpace>,
    V: VmInterface<ResourceId = ObjectId, Ownership = Lock>,
> {
    exec_ctx: &'e mut TransactionContext<'a, 'b, S, V>,
    program: Address,
}

impl<
    'e,
    'a,
    'b,
    S: Store<StateSpace = StateSpace>,
    V: VmInterface<ResourceId = ObjectId, Ownership = Lock>,
> ProgramContext<'e, 'a, 'b, S, V>
{
    pub fn new(exec_ctx: &'e mut TransactionContext<'a, 'b, S, V>, program: Address) -> Self {
        Self { exec_ctx, program }
    }

    pub fn borrow(&self, _address: Address) -> &[u8] {
        // self.exec_ctx.borrow(address);
        &[]
    }

    pub fn borrow_mut(&mut self, _address: Address) -> &mut [u8] {
        &mut []
    }
}
