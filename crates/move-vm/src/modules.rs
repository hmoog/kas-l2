use indexmap::IndexMap;
use kas_l2_move_utils::CompiledModules;
use move_core_types::language_storage::ModuleId;

#[derive(Default)]
pub struct Modules {
    modules: IndexMap<ModuleId, Vec<u8>>,
}

impl Modules {
    pub fn add(&mut self, id: ModuleId, compiled_bytes: Vec<u8>) {
        self.modules.insert(id, compiled_bytes);
    }

    pub fn id(&self, index: usize) -> ModuleId {
        self.modules.get_index(index).unwrap().0.clone()
    }
}

impl From<CompiledModules> for Modules {
    fn from(modules: CompiledModules) -> Self {
        let serialized_modules = modules.into_iter().map(|(id, m)| (id, m.serialize()));
        Self {
            modules: IndexMap::from_iter(serialized_modules),
        }
    }
}

mod foreign_traits {
    use move_core_types::{
        identifier::IdentStr, language_storage::ModuleId, resolver, resolver::LinkageResolver,
    };

    use crate::Modules;

    impl LinkageResolver for Modules {
        type Error = String;

        fn link_context(&self) -> move_core_types::account_address::AccountAddress {
            move_core_types::account_address::AccountAddress::ZERO
        }

        fn relocate(&self, module_id: &ModuleId) -> Result<ModuleId, Self::Error> {
            Ok(module_id.clone())
        }

        fn defining_module(
            &self,
            module_id: &ModuleId,
            _struct: &IdentStr,
        ) -> Result<ModuleId, Self::Error> {
            Ok(module_id.clone())
        }
    }

    impl resolver::ModuleResolver for Modules {
        type Error = String;

        fn get_module(&self, id: &ModuleId) -> Result<Option<Vec<u8>>, Self::Error> {
            self.modules
                .get(id)
                .cloned()
                .ok_or("".to_string())
                .map(Some)
        }
    }
}
