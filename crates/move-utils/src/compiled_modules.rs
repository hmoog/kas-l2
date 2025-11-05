use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::PathBuf,
    str::FromStr,
};

use move_compiler::compiled_unit::{AnnotatedCompiledModule, CompiledUnit};
use move_core_types::language_storage::ModuleId;
use tempfile::TempDir;

pub struct CompiledModules {
    compiled_modules: HashMap<ModuleId, CompiledUnit>,
}

impl CompiledModules {
    pub fn from_sources(sources: &[&str], dependencies: &[&str]) -> Self {
        Self {
            compiled_modules: Self::compile_sources(sources, dependencies)
                .into_iter()
                .map(|s| (s.module_id().1, s.into_compiled_unit()))
                .collect(),
        }
    }

    pub fn serialize(&self, module_ids: Vec<&str>) -> Result<Vec<Vec<u8>>, anyhow::Error> {
        let mut serialized_modules = Vec::with_capacity(module_ids.len());
        for module_id in module_ids {
            serialized_modules.push(
                self.compiled_modules
                    .get(&ModuleId::from_str(module_id)?)
                    .map(|unit| unit.serialize())
                    .ok_or_else(|| anyhow::anyhow!("module not found: {}", module_id))?,
            );
        }

        Ok(serialized_modules)
    }

    fn compile_sources(sources: &[&str], dependencies: &[&str]) -> Vec<AnnotatedCompiledModule> {
        let dir = TempDir::new().unwrap();
        let source_files = Self::write_temp_files(dir.path().join("src"), sources);
        let dep_files = Self::write_temp_files(dir.path().join("deps"), dependencies);

        move_compiler::Compiler::from_files::<String, String>(
            None,
            source_files,
            dep_files,
            BTreeMap::new(),
        )
        .build()
        .expect("failed to build")
        .1
        .expect("failed to build")
        .0
    }

    fn write_temp_files(prefix: PathBuf, sources: &[&str]) -> Vec<String> {
        fs::create_dir_all(&prefix).expect("failed to create directory");

        let mut temp_files = Vec::with_capacity(sources.len());
        for (i, src) in sources.iter().enumerate() {
            let virtual_source_path = prefix.join(i.to_string()).to_string_lossy().to_string();
            fs::write(&virtual_source_path, src).expect("failed to write file");
            temp_files.push(virtual_source_path);
        }
        temp_files
    }
}

impl IntoIterator for CompiledModules {
    type Item = (ModuleId, CompiledUnit);
    type IntoIter = std::collections::hash_map::IntoIter<ModuleId, CompiledUnit>;
    fn into_iter(self) -> Self::IntoIter {
        self.compiled_modules.into_iter()
    }
}
