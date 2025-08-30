use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use glob::glob;
use serde::Deserialize;
use serde_json::Value as Json;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use which::which;

#[derive(Parser)]
#[command(bin_name = "cargo-kas", version, about = "Dockerized builder/packager for kas artifacts")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Build artifacts; package into target/kas/<name>.kas
    BuildProgram {
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        skip_prove: bool,
        #[arg(long)]
        skip_sbf: bool,
        /// Force reproducible dockerized builds for both SBF and SP1 (ignore local toolchains)
        #[arg(long)]
        reproducible: bool,
        #[arg(long)]
        verbose: bool,
        #[arg(long, value_enum, default_value_t = LenPrefix::U64)]
        len: LenPrefix,
        #[arg(long, default_value = "solanafoundation/solana-verifiable-build:2.3.6")]
        solana_image: String,
        #[arg(long, default_value = "debian:bookworm-slim")]
        rust_image: String,
        #[arg(long)]
        out_dir: Option<PathBuf>,
    },
}

#[derive(Copy, Clone, Eq, PartialEq, ValueEnum)]
enum LenPrefix {
    U32,
    U64,
}

#[derive(Debug, Deserialize, Clone)]
struct CargoPackage {
    name: String,
    manifest_path: String,
    targets: Vec<CargoTarget>,
}

#[derive(Debug, Deserialize, Clone)]
struct CargoTarget {
    name: String,
    kind: Vec<String>,
    crate_types: Vec<String>,
}

fn main() -> Result<()> {
    // Support invoking as `cargo kas ...`
    let mut args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "kas" {
        args.remove(1);
    }
    let cli = Cli::parse_from(args);

    match cli.cmd {
        Cmd::BuildProgram {
            name,
            skip_prove,
            skip_sbf,
            reproducible,
            verbose,
            len,
            solana_image,
            rust_image,
            out_dir,
        } => build_program(
            name,
            skip_prove,
            skip_sbf,
            reproducible,
            verbose,
            len,
            &solana_image,
            &rust_image,
            out_dir,
        ),
    }
}

fn build_program(
    name_cli: Option<String>,
    skip_prove: bool,
    skip_sbf: bool,
    reproducible: bool,
    verbose: bool,
    len: LenPrefix,
    solana_image: &str,
    rust_image: &str,
    out_dir_cli: Option<PathBuf>,
) -> Result<()> {
    // Track whether we touched Docker so we can fix ownership at the end.
    let mut used_docker = false;

    // ---- workspace / metadata ----
    let meta = cargo_metadata_json()?;
    let workspace_root = PathBuf::from(meta["workspace_root"].as_str().unwrap_or("."))
        .canonicalize()
        .unwrap_or(env::current_dir()?);
    let cwd = env::current_dir()?.canonicalize()?;
    let (primary_pkg, _) = select_primary_package(&meta)?;
    let rel_cwd = cwd.strip_prefix(&workspace_root).unwrap_or(Path::new(""));
    let rel_cwd_str = rel_cwd.to_string_lossy().to_string();
    let out_name = name_cli.unwrap_or_else(|| primary_pkg.name.clone());

    // Expected artifacts
    let (expected_sbf, sp1_bins) = derive_expected_artifacts(&primary_pkg.targets);

    if verbose {
        eprintln!("[kas] package: {}", primary_pkg.name);
        eprintln!("[kas] workspace_root: {}", workspace_root.display());
        eprintln!("[kas] cwd: {}", cwd.display());
        eprintln!("[kas] rel_cwd: {}", rel_cwd_str);
        if !expected_sbf.is_empty() {
            eprintln!("[kas] expected SBF artifacts:");
            for f in &expected_sbf {
                eprintln!("   - {f}");
            }
        }
        if !sp1_bins.is_empty() {
            eprintln!("[kas] expected SP1 bin names:");
            for b in &sp1_bins {
                eprintln!("   - {b}");
            }
        }
        if reproducible {
            eprintln!("[kas] reproducible mode: forcing dockerized builds for SBF and SP1");
        }
    }

    // ---- dirs ----
    let out_dir = out_dir_cli.unwrap_or_else(|| PathBuf::from("target/kas"));
    let stage_dir = out_dir.join("_staging").join(&out_name);
    fs::create_dir_all(&stage_dir)?;
    fs::create_dir_all(&out_dir)?;
    let out_path = out_dir.join(format!("{out_name}.kas"));

    // ---- build SBF (local preferred unless reproducible) ----
    if !skip_sbf {
        used_docker |= build_sbf(
            &workspace_root,
            &rel_cwd_str,
            reproducible,
            verbose,
            solana_image,
        )?;
    } else {
        vlog(verbose, "Solana: skipped");
    }

    // ---- stage SBF artifacts ----
    let mut staged: Vec<PathBuf> = Vec::new();
    let sbf_roots = [workspace_root.join("target/sbpf-solana-solana/release")];
    if verbose {
        eprintln!("[kas] SBF artifact dir:");
        for r in &sbf_roots {
            eprintln!("   - {}", r.display());
        }
    }
    stage_expected(&expected_sbf, &sbf_roots, &stage_dir, &mut staged, verbose)?;

    // ---- build SP1 (local preferred unless reproducible) ----
    if !skip_prove {
        used_docker |= build_sp1(
            &workspace_root,
            &rel_cwd_str,
            reproducible,
            verbose,
            rust_image,
        )?;
    } else {
        vlog(verbose, "SP1: skipped");
    }

    // ---- stage SP1 artifacts (bins only) ----
    let sp1_release_dir =
        workspace_root.join("target/elf-compilation/riscv32im-succinct-zkvm-elf/release");
    if verbose {
        eprintln!("[kas] SP1 release dir:\n   - {}", sp1_release_dir.display());
    }
    stage_sp1_bins(&sp1_bins, &sp1_release_dir, &stage_dir, &mut staged, verbose)?;

    if staged.is_empty() {
        bail!("no artifacts found in {}", stage_dir.display());
    }

    // ---- pack artifacts ----
    staged.sort();
    let f = fs::File::create(&out_path)?;
    let mut w = BufWriter::new(f);

    for p in &staged {
        let bytes = fs::read(p)?;
        match len {
            LenPrefix::U32 => {
                let n = u32::try_from(bytes.len()).context("too large")?;
                w.write_all(&n.to_le_bytes())?;
            }
            LenPrefix::U64 => {
                let n = u64::try_from(bytes.len()).unwrap();
                w.write_all(&n.to_le_bytes())?;
            }
        }
        w.write_all(&bytes)?;
    }
    w.flush()?;

    println!("Wrote {} artifacts -> {}", staged.len(), out_path.display());
    for p in staged {
        println!("  - {}", p.display());
    }

    // ---- finalizer: fix ownership if docker touched the tree ----
    if used_docker {
        if let Err(e) =
            docker_fix_ownership(&env::current_dir()?.canonicalize()?, rust_image, verbose)
        {
            eprintln!("[kas] warning: failed to fix ownership after docker build: {e}");
        }
    }

    Ok(())
}

/* ---------- build helpers ---------- */

fn build_sbf(
    workspace_root: &Path,
    rel_cwd: &str,
    reproducible: bool,
    verbose: bool,
    solana_image: &str,
) -> Result<bool> {
    let mut used_docker = false;

    if !reproducible {
        match which("cargo-build-sbf") {
            Ok(_) => {
                vlog(verbose, "Solana: cargo-build-sbf (local)");
                let status = Command::new("cargo-build-sbf")
                    .args(["--", "--lib"])
                    .stdin(Stdio::null())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status();

                match status {
                    Ok(s) if s.success() => return Ok(false),
                    Ok(s) => bail!("local cargo-build-sbf failed (exit code {:?})", s.code()),
                    Err(err) => {
                        eprintln!(
                            "[kas] local 'cargo-build-sbf' could not be executed: {err}\n\
                             [kas] Falling back to Docker-based SBF build.\n\
                             [kas] Tip: install Solana CLI (provides 'cargo-build-sbf') for faster builds:\n\
                             [kas]   https://docs.solana.com/cli/install-solana-cli-tools"
                        );
                    }
                }
            }
            Err(_) => {
                eprintln!(
                    "[kas] 'cargo-build-sbf' not found locally; falling back to Docker-based SBF build.\n\
                     [kas] Tip: install Solana CLI (provides 'cargo-build-sbf') for faster builds:\n\
                     [kas]   https://docs.solana.com/cli/install-solana-cli-tools"
                );
            }
        }
    } else {
        vlog(verbose, "Solana: reproducible mode -> using Docker");
    }

    which("docker").context("docker is required for SBF build")?;
    used_docker = true;

    let build_cmd = if rel_cwd.is_empty() {
        "cargo-build-sbf -- --lib".to_string()
    } else {
        format!("cd {rel_cwd} && cargo-build-sbf -- --lib")
    };

    docker_run(
        &[
            "run",
            "--rm",
            "-v",
            &format!("{}:/work", workspace_root.display()),
            "-w",
            "/work",
            solana_image,
            "bash",
            "-lc",
            &format!(
                "set -euo pipefail; \
                 export PATH=/usr/local/cargo/bin:/root/.local/share/solana/install/active_release/bin:$PATH; \
                 {build_cmd}"
            ),
        ],
        verbose,
    )?;

    Ok(used_docker)
}

fn build_sp1(
    workspace_root: &Path,
    rel_cwd: &str,
    reproducible: bool,
    verbose: bool,
    rust_image: &str,
) -> Result<bool> {
    let mut used_docker = false;

    if !reproducible {
        match which("cargo-prove") {
            Ok(_) => {
                vlog(verbose, "SP1: cargo prove build (local)");
                let status = Command::new("cargo")
                    .args(["prove", "build"])
                    .stdin(Stdio::null())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status();

                match status {
                    Ok(s) if s.success() => return Ok(false),
                    Ok(s) => bail!("local `cargo prove build` failed (exit code {:?})", s.code()),
                    Err(err) => {
                        eprintln!(
                            "[kas] local 'cargo prove build' could not be executed: {err}\n\
                             [kas] Falling back to Docker-based SP1 build.\n\
                             [kas] Tip: install SP1 locally for faster builds:\n\
                             [kas]   curl -sSL https://sp1up.succinct.xyz | bash && sp1up"
                        );
                    }
                }
            }
            Err(_) => {
                eprintln!(
                    "[kas] 'cargo prove' (cargo-prove) not found locally; falling back to Docker-based SP1 build.\n\
                     [kas] Tip: install SP1 locally for faster builds:\n\
                     [kas]   curl -sSL https://sp1up.succinct.xyz | bash && sp1up"
                );
            }
        }
    } else {
        vlog(verbose, "SP1: reproducible mode -> using Docker");
    }

    which("docker").context("docker is required for SP1 build")?;
    used_docker = true;

    let inner = if rel_cwd.is_empty() {
        "cargo prove build".to_string()
    } else {
        format!("cd {rel_cwd} && cargo prove build")
    };

    let script = format!(
        "set -euo pipefail; \
         apt-get update -qq; apt-get install -y -qq curl git ca-certificates build-essential; \
         if ! command -v cargo >/dev/null 2>&1; then curl https://sh.rustup.rs -sSf | sh -s -- -y; fi; \
         . $HOME/.cargo/env; \
         export PATH=$HOME/.cargo/bin:$PATH; \
         curl -sSL https://sp1up.succinct.xyz | bash; \
         source /root/.bashrc; \
         sp1up; \
         {inner}"
    );

    docker_run(
        &[
            "run",
            "--rm",
            "-v",
            &format!("{}:/work", workspace_root.display()),
            "-w",
            "/work",
            rust_image,
            "bash",
            "-lc",
            &script,
        ],
        verbose,
    )?;

    Ok(used_docker)
}

/* ---------- metadata & selection ---------- */

fn cargo_metadata_json() -> Result<Json> {
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

fn select_primary_package(meta: &Json) -> Result<(CargoPackage, Vec<CargoPackage>)> {
    let cwd = env::current_dir()?.canonicalize()?;
    let pkgs_val = meta["packages"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let mut pkgs: Vec<CargoPackage> = Vec::with_capacity(pkgs_val.len());
    for p in pkgs_val {
        pkgs.push(serde_json::from_value(p)?);
    }

    let workspace_root = PathBuf::from(meta["workspace_root"].as_str().unwrap_or("."));
    let root_manifest = workspace_root.join("Cargo.toml");

    // Prefer workspace root manifest package
    if let Some(p) = pkgs.iter().find(|p| Path::new(&p.manifest_path) == root_manifest) {
        return Ok((p.clone(), pkgs));
    }
    // Otherwise prefer current directory package
    let cwd_manifest = cwd.join("Cargo.toml");
    if let Some(p) = pkgs.iter().find(|p| Path::new(&p.manifest_path) == cwd_manifest) {
        return Ok((p.clone(), pkgs));
    }
    // Fallback to first package
    let primary = pkgs.first().cloned().expect("no packages");
    Ok((primary, pkgs))
}

/* ---------- artifact discovery & staging ---------- */

// Return (expected_sbf_files, sp1_bin_names)
fn derive_expected_artifacts(targets: &Vec<CargoTarget>) -> (Vec<String>, Vec<String>) {
    let mut sbf = Vec::new();
    let mut sp1_bins = Vec::new();
    let mut seen = HashSet::new();

    for t in targets {
        // Skip Cargo build-script target
        if t.name == "build_script_build" || t.kind.iter().any(|k| k == "custom-build") {
            continue;
        }

        let name = t.name.replace('-', "_");
        let has = |s: &str| t.kind.iter().any(|k| k == s) || t.crate_types.iter().any(|c| c == s);

        // SBF shared objects
        if has("cdylib") || has("dylib") {
            let cand = format!("{name}.so");
            if seen.insert(format!("sbf::{cand}")) {
                sbf.push(cand);
            }
        }

        // SP1 bins
        if has("bin") && seen.insert(format!("bin::{name}")) {
            sp1_bins.push(name);
        }
    }

    (sbf, sp1_bins)
}

fn stage_copy_unique(
    src: &Path,
    stage_dir: &Path,
    staged: &mut Vec<PathBuf>,
    verbose: bool,
) -> Result<()> {
    let dest = stage_dir.join(src.file_name().unwrap());
    if dest.exists() {
        if !dest.is_file() {
            bail!("staging destination exists and is not a file: {}", dest.display());
        }
        // Count pre-existing staged artifacts so packaging works on repeat runs.
        staged.push(dest.clone());
        vlog(
            verbose,
            &format!("    -> already staged {}, counting existing", dest.display()),
        );
        return Ok(());
    }

    fs::copy(src, &dest)?;
    staged.push(dest.clone());
    vlog(verbose, &format!("    -> staged {}", src.display()));
    Ok(())
}

fn stage_expected(
    expected_filenames: &Vec<String>,
    roots: &[PathBuf],
    stage_dir: &Path,
    staged: &mut Vec<PathBuf>,
    verbose: bool,
) -> Result<()> {
    for fname in expected_filenames {
        let mut found = false;
        vlog(verbose, &format!("searching for expected SBF file: {fname}"));

        // Direct lookups
        for r in roots {
            let p = r.join(fname);
            vlog(verbose, &format!("  - check {}", p.display()));
            if p.is_file() {
                stage_copy_unique(&p, stage_dir, staged, verbose)?;
                found = true;
                break;
            } else {
                vlog(verbose, "    -> not found here");
            }
        }
        if found {
            continue;
        }

        // Glob fallback within each root
        for r in roots {
            let pat = format!("{}/**/{}", r.display(), fname);
            vlog(verbose, &format!("  - glob {pat}"));
            for entry in glob(&pat)? {
                if let Ok(p) = entry {
                    if p.is_file() {
                        stage_copy_unique(&p, stage_dir, staged, verbose)?;
                        found = true;
                        break;
                    }
                }
            }
            if found {
                break;
            }
        }

        if !found {
            vlog(verbose, &format!("  ! {} not found in any root", fname));
        }
    }
    Ok(())
}

// Only check the canonical SP1 release directory and the bin name (no .elf fallback)
fn stage_sp1_bins(
    bin_names: &Vec<String>,
    release_dir: &Path,
    stage_dir: &Path,
    staged: &mut Vec<PathBuf>,
    verbose: bool,
) -> Result<()> {
    if bin_names.is_empty() {
        return Ok(());
    }
    for name in bin_names {
        let candidate = release_dir.join(name);
        vlog(verbose, &format!("SP1 check {}", candidate.display()));
        if candidate.is_file() {
            stage_copy_unique(&candidate, stage_dir, staged, verbose)?;
        } else {
            vlog(verbose, &format!("  ! SP1 binary not found: {}", candidate.display()));
        }
    }
    Ok(())
}

/* ---------- docker helpers ---------- */

fn docker_run(args: &[&str], verbose: bool) -> Result<()> {
    let mut cmd = Command::new("docker");
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    vlog(verbose, &format!("docker {}", args.join(" ")));
    let status = cmd.status()?;
    if !status.success() {
        bail!("docker command failed");
    }
    Ok(())
}

// Final ownership fix: use uid:gid derived from the /work mount, always try at the end.
fn docker_fix_ownership(workspace_root: &Path, image: &str, verbose: bool) -> Result<()> {
    which("docker").context("docker is required to fix ownership after docker builds")?;
    let script = "set -eu; OWNER=$(stat -c '%u:%g' /work 2>/dev/null || echo 0:0); chown -R \"$OWNER\" /work";
    docker_run(
        &[
            "run",
            "--rm",
            "-v",
            &format!("{}:/work", workspace_root.display()),
            "-w",
            "/work",
            image, // Debian-based; has /bin/sh + coreutils
            "sh",
            "-lc",
            script,
        ],
        verbose,
    )
}

fn vlog(verbose: bool, msg: &str) {
    if verbose {
        eprintln!("[kas] {msg}");
    }
}
