use std::{collections::BTreeMap, fs, path::PathBuf};

use move_compiler::compiled_unit::AnnotatedCompiledModule;
use tempfile::TempDir;

pub struct Compiler;

impl Compiler {
    pub fn compile_sources(srcs: &[&str], deps: &[&str]) -> Vec<AnnotatedCompiledModule> {
        let dir = TempDir::new().unwrap();
        let source_files = Self::write_temp_files(dir.path().join("src"), srcs);
        let dep_files = Self::write_temp_files(dir.path().join("deps"), deps);

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
