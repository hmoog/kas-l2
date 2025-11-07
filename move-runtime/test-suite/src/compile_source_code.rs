use std::{collections::BTreeMap, sync::Arc};

use move_compiler::{
    Compiler, PreCompiledProgramInfo, compiled_unit::AnnotatedCompiledUnit,
    construct_pre_compiled_lib,
};
use move_core_types::parsing::address::NumericalAddress;
use move_symbol_pool::Symbol;
use vfs::{VfsPath, impls::memory::MemoryFS};

pub fn compile_source_code(sources: &[&str]) -> Arc<PreCompiledProgramInfo> {
    let vfs_root = VfsPath::new(MemoryFS::new());
    let source_files = write_vfs_files(&vfs_root, "deps", sources);

    let source_packages = vec![move_compiler::shared::PackagePaths {
        name: None,
        paths: source_files,
        named_address_map: BTreeMap::<Symbol, NumericalAddress>::new(),
    }];

    let result = construct_pre_compiled_lib(
        source_packages,
        None,
        None,
        false,
        move_compiler::Flags::empty(),
        Some(vfs_root),
    )
    .unwrap();

    Arc::new(result.unwrap())
}

pub fn compile_source_code_with_deps(
    sources: &[&str],
    deps: Arc<PreCompiledProgramInfo>,
) -> Vec<AnnotatedCompiledUnit> {
    let vfs_root = VfsPath::new(MemoryFS::new());
    let source_files = write_vfs_files(&vfs_root, "src", sources);

    let (_files, units_res) = Compiler::from_files(
        Some(vfs_root),
        source_files,
        vec![], // No source deps needed
        BTreeMap::<Symbol, NumericalAddress>::new(),
    )
    .set_pre_compiled_program_opt(Some(deps))
    .build()
    .expect("failed to compile");

    units_res.expect("compilation errors").0
}

fn write_vfs_files(vfs_root: &VfsPath, prefix: &str, sources: &[&str]) -> Vec<Symbol> {
    sources
        .iter()
        .enumerate()
        .map(|(i, src)| {
            // Create a virtual file path like "src/0.move"
            let file_path = format!("{}/{}.move", prefix, i);
            let vfs_path = vfs_root.join(&file_path).unwrap();

            // Write the source string to the virtual file
            vfs_path.parent().create_dir_all().unwrap();
            vfs_path.create_file().unwrap().write_all(src.as_bytes()).unwrap();

            // Return the path as a Symbol (not the source text)
            Symbol::from(file_path)
        })
        .collect()
}
