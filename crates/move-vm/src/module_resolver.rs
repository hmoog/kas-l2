use std::{
    collections::{BTreeMap, HashMap},
    fs,
};
use indexmap::IndexMap;
use move_compiler::Compiler;
use move_core_types::{identifier::IdentStr, language_storage::ModuleId, resolver, resolver::{LinkageResolver}};
use tempfile::TempDir;

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
        let dir = TempDir::new().unwrap();
        let mut temp_files = Vec::with_capacity(sources.len());
        for (i, src) in sources.iter().enumerate() {
            let virtual_source_path = dir.path().join(i.to_string()).to_string_lossy().to_string();
            fs::write(&virtual_source_path, src).expect("failed to write source file");
            temp_files.push(virtual_source_path);
        }

        let compiled_units =
            match Compiler::from_files::<String, String>(None, temp_files, vec![], BTreeMap::new())
                .build()
            {
                Ok((_, result)) => match result {
                    Ok((units, _)) => units,
                    Err(diagnostics) => panic!("Compilation failed: {:?}", diagnostics),
                },
                Err(err) => panic!("Compilation failed: {}", err),
            };

        let mut objects = IndexMap::new();
        for compiled_units in compiled_units {
            objects.insert(
                compiled_units.module_id().1,
                compiled_units.into_compiled_unit().serialize(),
            );
        }

        Self { objects }
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
