use std::collections::{HashMap, HashSet};

use borsh::BorshDeserialize;
use vprogs_scheduling_scheduler::{AccessHandle, VmInterface};
use vprogs_scheduling_types::AccessMetadata;
use vprogs_storage_state::StateSpace;
use vprogs_storage_types::Store;
use vprogs_transaction_runtime_address::Address;
use vprogs_transaction_runtime_authenticated_data::AuthenticatedData;
use vprogs_transaction_runtime_data::Data;
use vprogs_transaction_runtime_error::{VmError, VmResult};
use vprogs_transaction_runtime_instruction::Instruction;
use vprogs_transaction_runtime_lock::Lock;
use vprogs_transaction_runtime_object_id::ObjectId;
use vprogs_transaction_runtime_program::Program;
use vprogs_transaction_runtime_pubkey::PubKey;
use vprogs_transaction_runtime_transaction::Transaction;
use vprogs_transaction_runtime_transaction_effects::TransactionEffects;

pub struct TransactionRuntime<'a, 'b, S, V>
where
    S: Store<StateSpace = StateSpace>,
    V: VmInterface<ResourceId = ObjectId, Ownership = Lock>,
{
    handles: &'a mut [AccessHandle<'b, S, V>],
    signers: HashSet<PubKey>,
    loaded_data: HashMap<Address, AuthenticatedData>,
    loaded_programs: HashMap<Address, Program>,
}

impl<'a, 'b, S, V> TransactionRuntime<'a, 'b, S, V>
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

mod auth_context;
mod data_context;
