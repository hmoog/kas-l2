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
    /// Build artifacts in Docker; package into target/kas/<name>.kas
    BuildProgram {
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        skip_prove: bool,
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
            verbose,
            len,
            solana_image,
            rust_image,
            out_dir,
        } => build_program(
            name,
            skip_prove,
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
    verbose: bool,
    len: LenPrefix,
    solana_image: &str,
    rust_image: &str,
    out_dir_cli: Option<PathBuf>,
) -> Result<()> {
    which("docker").context("docker is required on the host")?;

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
    let targets = &primary_pkg.targets;
    let expected = derive_expected_filenames(targets);

    if verbose {
        eprintln!("[kas] package: {}", primary_pkg.name);
        eprintln!("[kas] workspace_root: {}", workspace_root.display());
        eprintln!("[kas] cwd: {}", cwd.display());
        eprintln!("[kas] rel_cwd: {}", rel_cwd_str);
        eprintln!("[kas] expected artifacts:");
        for f in &expected {
            eprintln!("   - {f}");
        }
    }

    // ---- dirs ----
    let out_dir = out_dir_cli.unwrap_or_else(|| PathBuf::from("target/kas"));
    let stage_dir = out_dir.join("_staging").join(&out_name);
    fs::create_dir_all(&stage_dir)?;
    fs::create_dir_all(&out_dir)?;
    let out_path = out_dir.join(format!("{out_name}.kas"));

    // ---- Solana build (always verifiable) ----
    vlog(verbose, "Solana: cargo-build-sbf");
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

    // ---- Stage Solana artifacts ----
    let mut staged: Vec<PathBuf> = Vec::new();
    let solana_search_roots = [
        PathBuf::from("target/verifiable"),
        PathBuf::from("target/deploy"),
        PathBuf::from("target/sbf-solana-solana/release"),
        PathBuf::from("target/sbpf-solana-solana/release"),
    ];
    stage_expected(&expected, &solana_search_roots, &stage_dir, &mut staged, verbose)?;

    // ---- SP1 build ----
    if !skip_prove {
        vlog(verbose, "SP1: cargo prove build --docker");
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

        let bin_names: Vec<String> = targets
            .iter()
            .filter(|t| t.kind.iter().any(|k| k == "bin"))
            .map(|t| t.name.clone())
            .collect();
        let sp1_search_roots = [PathBuf::from("target"), PathBuf::from(".")];
        stage_sp1_elves(&bin_names, &sp1_search_roots, &stage_dir, &mut staged, verbose)?;
    } else {
        vlog(verbose, "SP1: skipped");
    }

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

fn derive_expected_filenames(targets: &Vec<CargoTarget>) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for t in targets {
        let name = t.name.replace('-', "_");
        let has = |s: &str| t.kind.iter().any(|k| k == s) || t.crate_types.iter().any(|c| c == s);
        if has("cdylib") || has("dylib") {
            for cand in [format!("{name}.so"), format!("lib{name}.so")] {
                if seen.insert(cand.clone()) {
                    out.push(cand);
                }
            }
        }
        if has("lib") || has("rlib") {
            let f = format!("lib{name}.rlib");
            if seen.insert(f.clone()) {
                out.push(f);
            }
        }
        if has("bin") {
            if seen.insert(name.clone()) {
                out.push(name.clone());
            }
            let elf = format!("{name}.elf");
            if seen.insert(elf.clone()) {
                out.push(elf);
            }
        }
    }
    out
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
        for r in roots {
            let p = r.join(fname);
            if p.is_file() {
                let dest = stage_dir.join(p.file_name().unwrap());
                fs::copy(&p, &dest)?;
                staged.push(dest);
                vlog(verbose, &format!("staged {}", p.display()));
                found = true;
                break;
            }
        }
        if !found {
            for r in roots {
                let pat = format!("{}/**/{}", r.display(), fname);
                for entry in glob(&pat)? {
                    if let Ok(p) = entry {
                        if p.is_file() {
                            let dest = stage_dir.join(p.file_name().unwrap());
                            fs::copy(&p, &dest)?;
                            staged.push(dest);
                            vlog(verbose, &format!("staged {}", p.display()));
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
    }
    Ok(())
}

fn stage_sp1_elves(
    bin_names: &Vec<String>,
    roots: &[PathBuf],
    stage_dir: &Path,
    staged: &mut Vec<PathBuf>,
    verbose: bool,
) -> Result<()> {
    let mut want: HashSet<String> = bin_names.iter().map(|n| format!("{n}.elf")).collect();
    if want.is_empty() {
        for r in roots {
            let pat = format!("{}/**/release/*.elf", r.display());
            for entry in glob(&pat)? {
                if let Ok(p) = entry {
                    if p.is_file() {
                        let dest = stage_dir.join(p.file_name().unwrap());
                        fs::copy(&p, &dest)?;
                        staged.push(dest);
                        vlog(verbose, &format!("staged {}", p.display()));
                    }
                }
            }
        }
        return Ok(());
    }
    for r in roots {
        for name in bin_names {
            let pat = format!("{}/**/release/{}.elf", r.display(), name);
            for entry in glob(&pat)? {
                if let Ok(p) = entry {
                    if p.is_file() {
                        let dest = stage_dir.join(p.file_name().unwrap());
                        fs::copy(&p, &dest)?;
                        staged.push(dest);
                        vlog(verbose, &format!("staged {}", p.display()));
                        want.remove(&format!("{name}.elf"));
                    }
                }
            }
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
