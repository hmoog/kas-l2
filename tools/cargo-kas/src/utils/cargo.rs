use anyhow::{bail, Context};
use serde::Deserialize;
use serde_json::Value as Json;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn metadata_json() -> anyhow::Result<Json> {
    let out = Command::new("cargo")
        .args(["metadata", "--format-version=1", "--no-deps"])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .context("cargo metadata failed")?;
    if !out.status.success() {
        bail!("cargo metadata failed");
    }
    Ok(serde_json::from_slice(&out.stdout)?)
}

pub fn select_primary_package(meta: &Json) -> anyhow::Result<(CargoPackage, Vec<CargoPackage>)> {
    let cwd = env::current_dir()?.canonicalize()?;
    let pkgs_val = meta["packages"].as_array().cloned().unwrap_or_default();

    let mut pkgs: Vec<CargoPackage> = Vec::with_capacity(pkgs_val.len());
    for p in pkgs_val {
        pkgs.push(serde_json::from_value(p)?);
    }

    let workspace_root = PathBuf::from(meta["workspace_root"].as_str().unwrap_or("."));
    let root_manifest = workspace_root.join("Cargo.toml");

    // Prefer workspace root manifest package
    if let Some(p) = pkgs
        .iter()
        .find(|p| Path::new(&p.manifest_path) == root_manifest)
    {
        return Ok((p.clone(), pkgs));
    }
    // Otherwise prefer current directory package
    let cwd_manifest = cwd.join("Cargo.toml");
    if let Some(p) = pkgs
        .iter()
        .find(|p| Path::new(&p.manifest_path) == cwd_manifest)
    {
        return Ok((p.clone(), pkgs));
    }
    // Fallback to first package
    let primary = pkgs.first().cloned().expect("no packages");
    Ok((primary, pkgs))
}

#[derive(Debug, Deserialize, Clone)]
pub struct CargoPackage {
    pub name: String,
    pub manifest_path: String,
    pub targets: Vec<CargoTarget>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CargoTarget {
    pub name: String,
    pub kind: Vec<String>,
    pub crate_types: Vec<String>,
}
