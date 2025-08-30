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
    // ---- cargo metadata ----
    let meta = cargo_metadata_json()?;
    let workspace_root = PathBuf::from(meta["workspace_root"].as_str().unwrap_or("."));
    let workspace_root = workspace_root.canonicalize().unwrap_or(env::current_dir()?);
    let cwd = env::current_dir()?.canonicalize()?;
    let (primary_pkg, _pkgs) = select_primary_package(&meta)?;

    // compute relative subdir
    let rel_cwd = cwd.strip_prefix(&workspace_root).unwrap_or(Path::new(""));
    let rel_cwd_str = rel_cwd.to_string_lossy();

    let out_name = name_cli.unwrap_or_else(|| primary_pkg.name.clone());

    // Separate expected SBF (shared objects) from SP1 (bin names)
    let (expected_sbf, sp1_bins) = derive_expected_artifacts(&primary_pkg.targets);

    if verbose {
        eprintln!("[kas] package: {}", primary_pkg.name);
        eprintln!("[kas] workspace_root: {}", workspace_root.display());
        eprintln!("[kas] cwd: {}", cwd.display());
        eprintln!("[kas] rel_cwd: {}", rel_cwd_str);
        eprintln!("[kas] expected SBF artifacts:");
        for f in &expected_sbf {
            eprintln!("   - {f}");
        }
        if !sp1_bins.is_empty() {
            eprintln!("[kas] expected SP1 bin names:");
            for f in &sp1_bins {
                eprintln!("   - {f}");
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

    // ---- Solana build (local preferred unless --reproducible, then Docker) ----
    if !skip_sbf {
        let mut use_docker_for_sbf = reproducible;

        if !use_docker_for_sbf {
            // Try local cargo-build-sbf in the current working directory
            match which("cargo-build-sbf") {
                Ok(_) => {
                    vlog(verbose, "Solana: cargo-build-sbf (local)");
                    let mut cmd = Command::new("cargo-build-sbf");
                    cmd.args(["--", "--lib"])
                        .stdin(Stdio::null())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .current_dir(&cwd);
                    vlog(verbose, "running: cargo-build-sbf -- --lib");
                    match cmd.status() {
                        Ok(status) => {
                            if !status.success() {
                                // Local tool exists; treat failure as a real build error and do NOT fallback (unless reproducible requested).
                                bail!(
                                    "local cargo-build-sbf failed (exit code {:?})",
                                    status.code()
                                );
                            }
                        }
                        Err(err) => {
                            // Tool was found but couldn't execute (likely missing runtime deps). Fallback to Docker.
                            eprintln!(
                                "[kas] local 'cargo-build-sbf' could not be executed: {err}\n\
                                 [kas] Falling back to Docker-based SBF build.\n\
                                 [kas] Tip: install Solana CLI (provides 'cargo-build-sbf') for faster builds:\n\
                                 [kas]   https://docs.solana.com/cli/install-solana-cli-tools"
                            );
                            use_docker_for_sbf = true;
                        }
                    }
                }
                Err(_) => {
                    eprintln!(
                        "[kas] 'cargo-build-sbf' not found locally; falling back to Docker-based SBF build.\n\
                         [kas] Tip: install Solana CLI (provides 'cargo-build-sbf') for faster builds:\n\
                         [kas]   https://docs.solana.com/cli/install-solana-cli-tools"
                    );
                    use_docker_for_sbf = true;
                }
            }
        } else {
            vlog(verbose, "Solana: reproducible mode -> using Docker");
        }

        if use_docker_for_sbf {
            which("docker").context("docker is required for SBF build")?;
            vlog(verbose, "Solana: cargo-build-sbf (docker)");
            let build_cmd = if rel_cwd_str.is_empty() {
                "cargo-build-sbf -- --lib".to_string()
            } else {
                format!("cd {} && cargo-build-sbf -- --lib", rel_cwd_str)
            };
            eprintln!("[kas] build_cmd: {}", build_cmd);
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
                         {build_cmd}; \
                         chown -R $(id -u):$(id -g) /work"
                    ),
                ],
                verbose,
            )?;
        }
    } else {
        vlog(verbose, "Solana: skipped");
    }

    // ---- Stage ONLY SBF artifacts ----
    let mut staged: Vec<PathBuf> = Vec::new();
    // The SBF artifact is always here: target/sbpf-solana-solana/release/<package-name>.so
    let solana_search_roots = [workspace_root.join("target/sbpf-solana-solana/release")];
    if verbose {
        eprintln!("[kas] SBF artifact dir:");
        for r in &solana_search_roots {
            eprintln!("   - {}", r.display());
        }
    }
    stage_expected(&expected_sbf, &solana_search_roots, &stage_dir, &mut staged, verbose)?;

    // ---- SP1 build (local preferred unless --reproducible, then Docker) ----
    if !skip_prove {
        let mut use_docker_for_sp1 = reproducible;

        if !use_docker_for_sp1 {
            // Prefer local `cargo prove build` if `cargo-prove` is installed and runnable.
            match which("cargo-prove") {
                Ok(_) => {
                    vlog(verbose, "SP1: cargo prove build (local)");
                    let mut cmd = Command::new("cargo");
                    cmd.args(["prove", "build"])
                        .stdin(Stdio::null())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .current_dir(&cwd);
                    vlog(verbose, "running: cargo prove build");
                    match cmd.status() {
                        Ok(status) => {
                            if !status.success() {
                                // It ran but failed; surface the failure (don't mask with Docker).
                                bail!(
                                    "local `cargo prove build` failed (exit code {:?})",
                                    status.code()
                                );
                            }
                        }
                        Err(err) => {
                            // Could not execute (likely missing toolchain/deps); fallback to Docker.
                            eprintln!(
                                "[kas] local 'cargo prove build' could not be executed: {err}\n\
                                 [kas] Falling back to Docker-based SP1 build.\n\
                                 [kas] Tip: install SP1 locally for faster builds:\n\
                                 [kas]   curl -sSL https://sp1up.succinct.xyz | bash && sp1up"
                            );
                            use_docker_for_sp1 = true;
                        }
                    }
                }
                Err(_) => {
                    eprintln!(
                        "[kas] 'cargo prove' (cargo-prove) not found locally; falling back to Docker-based SP1 build.\n\
                         [kas] Tip: install SP1 locally for faster builds:\n\
                         [kas]   curl -sSL https://sp1up.succinct.xyz | bash && sp1up"
                    );
                    use_docker_for_sp1 = true;
                }
            }
        } else {
            vlog(verbose, "SP1: reproducible mode -> using Docker");
        }

        if use_docker_for_sp1 {
            which("docker").context("docker is required for SP1 build")?;

            vlog(verbose, "SP1: cargo prove build (docker)");
            let inner = if rel_cwd_str.is_empty() {
                "cargo prove build".to_string()
            } else {
                format!("cd {} && cargo prove build", rel_cwd_str)
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
                 {inner}; \
                 chown -R $(id -u):$(id -g) /work"
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
        }
    } else {
        vlog(verbose, "SP1: skipped");
    }

    // ---- Stage ONLY SP1 artifacts (bins) ----
    // The SP1 artifact is always here: target/elf-compilation/riscv32im-succinct-zkvm-elf/release/<bin.name>
    let sp1_release_dir =
        workspace_root.join("target/elf-compilation/riscv32im-succinct-zkvm-elf/release");

    if verbose {
        eprintln!("[kas] SP1 release dir:");
        eprintln!("   - {}", sp1_release_dir.display());
    }

    stage_sp1_bins(&sp1_bins, &sp1_release_dir, &stage_dir, &mut staged, verbose)?;

    if staged.is_empty() {
        bail!("no artifacts found in {}", stage_dir.display());
    }

    // ---- Pack artifacts ----
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
    Ok(())
}

// ---- helpers ----

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
    let pkgs_val = meta["packages"].as_array().unwrap_or(&vec![]).clone();
    let mut pkgs: Vec<CargoPackage> = Vec::new();
    for p in pkgs_val {
        pkgs.push(serde_json::from_value(p.clone())?);
    }
    let root_manifest =
        PathBuf::from(meta["workspace_root"].as_str().unwrap_or(".")).join("Cargo.toml");
    let mut primary: Option<CargoPackage> = None;
    for p in &pkgs {
        if Path::new(&p.manifest_path) == root_manifest {
            primary = Some(p.clone());
            break;
        }
    }
    if primary.is_none() {
        let cwd_manifest = cwd.join("Cargo.toml");
        for p in &pkgs {
            if Path::new(&p.manifest_path) == cwd_manifest {
                primary = Some(p.clone());
                break;
            }
        }
    }
    let primary = primary.unwrap_or_else(|| pkgs.first().expect("no packages").clone());
    Ok((primary, pkgs))
}

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

        // SBF shared objects only
        if has("cdylib") || has("dylib") {
            let cand = format!("{name}.so");
            if seen.insert(format!("sbf::{cand}")) {
                sbf.push(cand);
            }
        }

        // SP1 bins (stage later, separately)
        if has("bin") {
            if seen.insert(format!("bin::{name}")) {
                sp1_bins.push(name);
            }
        }
    }

    (sbf, sp1_bins)
}

fn stage_copy_unique(src: &Path, stage_dir: &Path, staged: &mut Vec<PathBuf>, verbose: bool) -> Result<()> {
    let dest = stage_dir.join(src.file_name().unwrap());
    if dest.exists() {
        if dest.is_file() {
            // Count pre-existing staged artifacts so packaging works on repeat runs.
            staged.push(dest.clone());
            vlog(verbose, &format!("    -> already staged {}, counting existing", dest.display()));
            return Ok(());
        } else {
            bail!("staging destination exists and is not a file: {}", dest.display());
        }
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
        if !found {
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
        }
        if !found {
            vlog(verbose, &format!("  ! {} not found in any root", fname));
        }
    }
    Ok(())
}

// Only check the single canonical SP1 release directory and the bin name (no .elf fallback)
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

fn docker_run(args: &[&str], verbose: bool) -> Result<()> {
    let mut cmd = Command::new("docker");
    cmd.args(args).stdin(Stdio::null()).stdout(Stdio::inherit()).stderr(Stdio::inherit());
    vlog(verbose, &format!("docker {}", args.join(" ")));
    let status = cmd.status()?;
    if !status.success() {
        bail!("docker command failed");
    }
    Ok(())
}

fn vlog(verbose: bool, msg: &str) {
    if verbose {
        eprintln!("[kas] {msg}");
    }
}
