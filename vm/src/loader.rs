use crate::builtin::abort::Abort;
use crate::builtin::cpi::CPI;
use crate::builtin::sol_alloc_free::SolAllocFree;
use crate::builtin::sol_log::SolLog;
use crate::builtin::sol_log_64::SolLog64;
use crate::builtin::sol_memcpy::SolMemcpy;
use crate::builtin::sol_panic::SolPanic;
use crate::errors::VMResult;
use crate::executable::Executable;
use crate::{Program, Prover, RuntimeContext};
use solana_sbpf::program::BuiltinProgram;
use solana_sbpf::vm::Config;
use std::fs;
use std::sync::Arc;

pub struct Loader {
    pub builtin_program: Arc<BuiltinProgram<RuntimeContext>>,
}

impl Loader {
    pub fn new(config: Config) -> Self {
        Self {
            builtin_program: Arc::new({
                let mut loader = BuiltinProgram::new_loader(config);

                loader.register_function("abort", Abort::vm).unwrap();
                loader.register_function("cpi", CPI::vm).unwrap();
                loader
                    .register_function("sol_alloc_free_", SolAllocFree::vm)
                    .unwrap();
                loader.register_function("sol_log_", SolLog::vm).unwrap();
                loader
                    .register_function("sol_log_64_", SolLog64::vm)
                    .unwrap();
                loader
                    .register_function("sol_memcpy_", SolMemcpy::vm)
                    .unwrap();
                loader
                    .register_function("sol_panic_", SolPanic::vm)
                    .unwrap();

                loader
            }),
        }
    }

    pub fn load_program(&self, id: [u8; 32], bytes: Vec<u8>) -> VMResult<Program> {
        let mut bytes = &bytes[..];

        Ok(Program {
            executable: Executable::new(
                id,
                Self::read_binary_segment(&mut bytes),
                &self.builtin_program,
            )?,
            prover: Prover::new(id, Self::read_binary_segment(&mut bytes)),
        })
    }

    pub fn load_program_file(&self, id: [u8; 32], path: &str) -> VMResult<Program> {
        self.load_program(id, fs::read(path)?)
    }

    fn read_binary_segment<'a>(cursor: &mut &'a [u8]) -> &'a [u8] {
        if cursor.len() < 8 {
            panic!("Truncated length header");
        }

        let (len_bytes, rest) = cursor.split_at(8);
        let len = u64::from_le_bytes(len_bytes.try_into().unwrap()) as usize;

        if rest.len() < len {
            panic!("Truncated segment: expected {}, have {}", len, rest.len());
        }

        let (segment, rest) = rest.split_at(len);
        *cursor = rest;
        segment
    }
}
