use crate::RuntimeContext;
use crate::builtin::abort::Abort;
use crate::builtin::cpi::CPI;
use crate::builtin::sol_alloc_free::SolAllocFree;
use crate::builtin::sol_log::SolLog;
use crate::builtin::sol_log_64::SolLog64;
use crate::builtin::sol_memcpy::SolMemcpy;
use crate::builtin::sol_panic::SolPanic;
use crate::program::Program;
use solana_sbpf::elf::Executable;
use solana_sbpf::program::BuiltinProgram;
use solana_sbpf::verifier::RequisiteVerifier;
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

    pub fn load_elf_file(&self, id: [u8; 32], path: &str) -> Program {
        self.load_elf_bytes(id, fs::read(path).expect("failed to file").as_slice())
    }

    pub fn load_elf_bytes(&self, id: [u8; 32], elf_bytes: &[u8]) -> Program {
        let executable =
            Executable::from_elf(elf_bytes, self.builtin_program.clone()).expect("load executable");
        executable
            .verify::<RequisiteVerifier>()
            .expect("verify elf");

        Program { id, executable }
    }
}
