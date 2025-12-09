use std::collections::{HashMap, HashSet};

use borsh::BorshDeserialize;
use kas_l2_runtime_manager::{AccessHandle, VmInterface};
use kas_l2_runtime_state::StateSpace;
use kas_l2_runtime_types::AccessMetadata;
use kas_l2_storage_types::Store;
use kas_l2_vm_address::Address;
use kas_l2_vm_auth_context::AuthContext;
use kas_l2_vm_authenticated_data::AuthenticatedData;
use kas_l2_vm_data::Data;
use kas_l2_vm_data_context::DataContext;
use kas_l2_vm_error::{VmError, VmResult};
use kas_l2_vm_instruction::Instruction;
use kas_l2_vm_lock::Lock;
use kas_l2_vm_object_id::ObjectId;
use kas_l2_vm_program::Program;
use kas_l2_vm_pubkey::PubKey;
use kas_l2_vm_transaction::Transaction;
use kas_l2_vm_transaction_effects::TransactionEffects;

pub struct TransactionContext<'a, 'b, S, V>
where
    S: Store<StateSpace = StateSpace>,
    V: VmInterface<ResourceId = ObjectId, Ownership = Lock>,
{
    handles: &'a mut [AccessHandle<'b, S, V>],
    signers: HashSet<PubKey>,
    loaded_data: HashMap<Address, AuthenticatedData>,
    loaded_programs: HashMap<Address, Program>,
}

impl<'a, 'b, S, V> TransactionContext<'a, 'b, S, V>
where
    S: Store<StateSpace = StateSpace>,
    V: VmInterface<ResourceId = ObjectId, Ownership = Lock>,
{
    pub fn execute(
        tx: &'a Transaction,
        handles: &'a mut [AccessHandle<'b, S, V>],
    ) -> VmResult<TransactionEffects> {
        let signers = HashSet::new();
        let loaded_data = HashMap::new();
        let loaded_programs = HashMap::new();
        let mut this = Self { handles, signers, loaded_data, loaded_programs };

        this.ingest_state()?;

        for instruction in tx.instructions() {
            this.execute_instruction(instruction)?;
        }

        this.finalize()
    }

    fn ingest_state(&mut self) -> VmResult<()> {
        for handle in self.handles.iter() {
            match handle.access_metadata().id() {
                // TODO: VALIDATE PROGRAM WITH VM?
                ObjectId::Program(address) => {
                    let program = Program::deserialize(&mut handle.state().data.as_slice())?;

                    self.loaded_programs.insert(address, program);
                }
                // TODO: VERIFY CHECKSUMS, ETC.
                ObjectId::Data(address) => {
                    let data = Data::deserialize(&mut handle.state().data.as_slice())?;
                    let mut_cap = handle.state().owner.unlock(self);

                    self.loaded_data.insert(address, AuthenticatedData::new(data, mut_cap));
                }
                // TODO: RETURN CORRECT ERROR
                ObjectId::Empty => return Err(VmError::Generic),
            }
        }
        Ok(())
    }

    fn execute_instruction(&mut self, instruction: &Instruction) -> VmResult<()> {
        match instruction {
            Instruction::PublishProgram { program_bytes } => {
                let _ = program_bytes;
                // TODO: CHECK BYTES
                // TODO: STORE PROGRAM
                // TODO: PUSH PROGRAM ID TO RETURN VALUES
            }
            Instruction::CallProgram { program_id, args } => {
                let _ = (program_id, args);
                // TODO: CREATE PROGRAM CONTEXT
                // RESOLVE ARGS
                // EXECUTE PROGRAM WITH INVOCATION CONTEXT
                // TODO: HANDLE RETURN VALUES / TEAR DOWN PROGRAM CONTEXT
            }
        }
        Ok(())
    }

    fn finalize(self) -> VmResult<TransactionEffects> {
        Ok(TransactionEffects {})
    }
}

impl<'a, 'b, S, V> AuthContext for TransactionContext<'a, 'b, S, V>
where
    S: Store<StateSpace = StateSpace>,
    V: VmInterface<ResourceId = ObjectId, Ownership = Lock>,
{
    fn has_signer(&self, pub_key: &PubKey) -> bool {
        self.signers.contains(pub_key)
    }
}

impl<'a, 'b, S, V> DataContext for TransactionContext<'a, 'b, S, V>
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
