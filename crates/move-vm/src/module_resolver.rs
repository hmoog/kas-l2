use indexmap::IndexMap;
use kas_l2_move_utils::Compiler;
use move_core_types::{
    identifier::IdentStr, language_storage::ModuleId, resolver, resolver::LinkageResolver,
};

pub struct ModuleResolver {
    objects: IndexMap<ModuleId, Vec<u8>>,
}

impl ModuleResolver {
    pub fn new() -> ModuleResolver {
        ModuleResolver {
            objects: IndexMap::new(),
        }
    }

    pub fn module_id(&self, index: usize) -> &ModuleId {
        self.objects.get_index(index).unwrap().0
    }

    pub fn add_module(&mut self, module_id: ModuleId, module_bytes: Vec<u8>) {
        self.objects.insert(module_id, module_bytes);
    }

    pub fn from_sources(sources: &[&str]) -> Self {
        Self {
            objects: Compiler::compile_sources(sources, &[])
                .into_iter()
                .map(|u| (u.module_id().1, u.into_compiled_unit().serialize()))
                .collect(),
        }
    }
}

impl LinkageResolver for ModuleResolver {
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

impl resolver::ModuleResolver for ModuleResolver {
    type Error = String;

    fn get_module(&self, id: &ModuleId) -> Result<Option<Vec<u8>>, Self::Error> {
        self.objects
            .get(id)
            .cloned()
            .ok_or("".to_string())
            .map(Some)
    }
}
