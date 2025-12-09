use kas_l2_vm_address::Address;
use kas_l2_vm_authenticated_data::AuthenticatedData;
use kas_l2_vm_data_context::DataContext;
use kas_l2_vm_error::{VmError, VmResult};

pub struct ProgramContext<'e, D: DataContext> {
    data: &'e mut D,
    program: Address,
}

impl<'e, D: DataContext> ProgramContext<'e, D> {
    pub fn new(data: &'e mut D, program: Address) -> Self {
        Self { data, program }
    }
}

impl<'e, D: DataContext> DataContext for ProgramContext<'e, D> {
    fn borrow(&mut self, address: Address) -> VmResult<&AuthenticatedData> {
        self.data.borrow(address)
    }

    fn borrow_mut(&mut self, address: Address) -> VmResult<&mut AuthenticatedData> {
        self.data.borrow_mut(address).and_then(|data| {
            match data.data().owning_program() == &self.program {
                true => Ok(data),
                false => Err(VmError::MissingMutCapability(address)),
            }
        })
    }
}
