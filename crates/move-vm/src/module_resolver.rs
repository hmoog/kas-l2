use indexmap::IndexMap;
use kas_l2_move_utils::CompiledModules;
use move_core_types::{
    identifier::IdentStr, language_storage::ModuleId, resolver, resolver::LinkageResolver,
};

pub struct ModuleResolver {
    objects: IndexMap<ModuleId, Vec<u8>>,
}

impl ModuleResolver {
    pub fn add_module(&mut self, module_id: ModuleId, module_bytes: Vec<u8>) {
        self.objects.insert(module_id, module_bytes);
    }

    pub fn id(&self, index: usize) -> ModuleId {
        self.objects.get_index(index).unwrap().0.clone()
    }
}

impl Default for ModuleResolver {
    fn default() -> Self {
        Self {
            objects: IndexMap::new(),
        }
    }
}

impl From<CompiledModules> for ModuleResolver {
    fn from(modules: CompiledModules) -> Self {
        let serialized_modules = modules.into_iter().map(|(id, m)| (id, m.serialize()));
        Self {
            objects: IndexMap::from_iter(serialized_modules),
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
