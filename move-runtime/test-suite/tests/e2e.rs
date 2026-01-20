extern crate core;

use std::{str::FromStr, sync::Arc};

use move_compiler::PreCompiledProgramInfo;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    runtime_value::{MoveStruct, MoveValue},
};
use tempfile::TempDir;
use vprogs_move_runtime_test_suite::{
    AssertTx, AssertTxEffect, AssertTxExt, SerializePreCompiledProgramInfoExt, compile_source_code,
};
use vprogs_move_runtime_vm::{
    Instruction,
    ObjectAccess::{Read, Write},
    ObjectId, Transaction, Vm,
};
use vprogs_scheduling_scheduler::{ExecutionConfig, Scheduler};
use vprogs_storage_manager::StorageConfig;
use vprogs_storage_rocksdb_store::{DefaultConfig, RocksDbStore};

#[test]
pub fn test_move_runtime() -> Result<(), anyhow::Error> {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    {
        let mut runtime = Scheduler::new(
            ExecutionConfig::default().with_vm(Vm::default()),
            StorageConfig::default()
                .with_store(RocksDbStore::<DefaultConfig>::open(temp_dir.path())),
        );

        let batch = runtime.schedule(vec![
            Transaction {
                accessed_resources: vec![Write(ObjectId::from_str("0x1::Test")?)],
                instruction: Instruction::PublishModules {
                    modules: test_modules().serialize(vec!["0x1::Test"])?,
                    sender: AccountAddress::from_str("0x1")?,
                },
            },
            Transaction {
                accessed_resources: vec![Read(ObjectId::from_str("0x1::Test")?)],
                instruction: Instruction::MethodCall {
                    module: 0,
                    fn_name: Identifier::from_str("get_value")?,
                    ty_args: vec![],
                    args: vec![],
                },
            },
        ]);

        batch.wait_processed_blocking().txs().assert(&[AssertTx::new(
            1,
            vec![AssertTxEffect::ReturnValue {
                reference: (0, 0),
                expected: MoveValue::Struct(MoveStruct(vec![MoveValue::U64(7)])),
            }],
        )]);

        runtime.shutdown();

        Ok(())
    }
}

fn test_modules() -> Arc<PreCompiledProgramInfo> {
    compile_source_code(&[r#"
        module 0x1::Test {
            // Define a resource type that we can pass around
            public struct Obj has key { value: u64 }

            public fun new_obj(): Obj {
                Obj { value: 0 }
            }

            public fun f(o: &mut Obj) {
                o.value = o.value + 1;
            }

            public fun get_value(): u64 {
                7
            }
        }
    "#])
}
