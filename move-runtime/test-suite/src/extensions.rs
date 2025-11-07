use std::{collections::BTreeMap, str::FromStr, sync::Arc};

use move_compiler::PreCompiledProgramInfo;
use move_core_types::language_storage::ModuleId;

pub trait CombinePreCompiledProgramInfoExt:
    Iterator<Item = Arc<PreCompiledProgramInfo>> + Sized
{
    fn combine(self) -> Arc<PreCompiledProgramInfo> {
        let mut combined_modules = BTreeMap::new();

        for dep in self {
            for (mod_ident, mod_info) in dep.iter() {
                combined_modules.insert(*mod_ident, mod_info.clone());
            }
        }

        Arc::new(PreCompiledProgramInfo::new(combined_modules))
    }
}

impl<I> CombinePreCompiledProgramInfoExt for I where I: Iterator<Item = Arc<PreCompiledProgramInfo>> {}

pub trait SerializePreCompiledProgramInfoExt {
    fn serialize(&self, module_ids: Vec<&str>) -> Result<Vec<Vec<u8>>, anyhow::Error>;
}

impl SerializePreCompiledProgramInfoExt for PreCompiledProgramInfo {
    fn serialize(&self, module_ids: Vec<&str>) -> Result<Vec<Vec<u8>>, anyhow::Error> {
        let mut serialized_modules = Vec::with_capacity(module_ids.len());

        for module_id_str in module_ids {
            let module_id = ModuleId::from_str(module_id_str)?;

            // Find the matching module in PreCompiledProgramInfo
            let module_bytes = self
                .iter()
                .find(|(mod_ident, _)| {
                    // Convert E::ModuleIdent to ModuleId for comparison
                    let addr = mod_ident.value.address.into_addr_bytes().into_inner();
                    let name = mod_ident.value.module.to_string();
                    module_id == ModuleId::new(addr, name.parse().unwrap())
                })
                .and_then(|(_, mod_info)| mod_info.compiled_unit.as_ref())
                .map(|unit| unit.named_module.serialize())
                .ok_or_else(|| anyhow::anyhow!("module not found: {}", module_id_str))?;

            serialized_modules.push(module_bytes);
        }

        Ok(serialized_modules)
    }
}
