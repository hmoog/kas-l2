mod instruction;
mod module_resolver;
mod object_access;
mod object_id;
mod transaction;
mod vm;

pub use instruction::*;
pub use module_resolver::*;
pub use object_access::*;
pub use object_id::*;
pub use transaction::*;
pub use vm::*;

#[cfg(test)]
mod test {
    use move_core_types::{
        account_address::AccountAddress,
        identifier::Identifier,
        language_storage::ModuleId,
        runtime_value::{MoveStruct, MoveValue},
    };
    use move_vm_runtime::move_vm::MoveVM;
    use move_vm_types::gas::UnmeteredGasMeter;

    use crate::module_resolver::ModuleResolver;

    #[test]
    pub fn test_move_vm() {
        let object_store = ModuleResolver::from_sources(&[
            r#"
            module 0x1::A {
                // Define a resource type that we can pass around
                public struct Obj has key { value: u64 }

                public fun f(o: &mut Obj) {
                    o.value = o.value + 1;
                }
            }
            "#,
            r#"
            module 0x1::B {
                use 0x1::A;

                public entry fun g(o: &mut A::Obj) {
                    A::f(o);
                }
            }
            "#,
        ]);

        let vm = MoveVM::new([]).unwrap();

        let mut session = vm.new_session(object_store);

        let res = session.execute_entry_function(
            &ModuleId::new(AccountAddress::ONE, Identifier::new("B").unwrap()),
            &Identifier::new("g").unwrap(),
            Vec::new(),
            vec![
                bcs::to_bytes(&MoveStruct::new(vec![
                    MoveValue::U64(42), // Only field: value
                ]))
                .unwrap(),
            ],
            &mut UnmeteredGasMeter,
        );

        eprintln!("Execution result: {:?}", res);

        assert!(res.is_ok(), "MoveVM execution failed: {:?}", res);

        let (result, _store) = session.finish();

        match result {
            Ok(change_set) => {
                eprintln!("ChangeSet: {:?}", change_set);
            }
            Err(e) => panic!("Session finish failed: {:?}", e),
        }
    }
}
