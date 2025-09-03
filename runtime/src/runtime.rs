use std::sync::Arc;
use solana_sbpf::program::BuiltinProgram;
use kas_l2_vm::{Config, VM};
use crate::builtin::abort::Abort;
use crate::builtin::cpi::CPI;
use crate::builtin::sol_alloc_free::SolAllocFree;
use crate::builtin::sol_log::SolLog;
use crate::builtin::sol_log_64::SolLog64;
use crate::builtin::sol_memcpy::SolMemcpy;
use crate::builtin::sol_panic::SolPanic;
use crate::RuntimeContext;

pub struct Runtime {
    pub vm: VM<RuntimeContext>,
}

impl Runtime {
    pub fn new(config: Config) -> Self {
        Self {
            vm: VM::new(Arc::new({
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
            })),
        }
    }
}