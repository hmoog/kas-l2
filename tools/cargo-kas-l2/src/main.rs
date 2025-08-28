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
#[command(name = "cargo-kas", version, about = "Dockerized builder/packager for kas artifacts")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Build artifacts in Docker; package into target/kas-l2/<name>.kas
    BuildProgram {
        /// Output base name; defaults to current package name from Cargo.toml
        #[arg(long)]
        name: Option<String>,

        /// Prefer Anchor verifiable build if Anchor.toml exists
        #[arg(long)]
        prefer_anchor: bool,

        /// Force generic Solana verifiable-build path (ignore Anchor.toml)
        #[arg(long)]
        force_generic: bool,

        /// Skip SP1 (cargo prove) build
        #[arg(long)]
        skip_prove: bool,

        /// Verbose logs
        #[arg(long)]
        verbose: bool,

        /// Length prefix type for concatenation (default: u64 little-endian)
        #[arg(long, value_enum, default_value_t = LenPrefix::U64)]
        len: LenPrefix,

        /// Anchor image tag (if used)
        #[arg(long, default_value = "v0.31.1")]
        anchor_tag: String,

        /// Generic Solana verifiable-build image
        #[arg(long, default_value = "solanafoundation/solana-verifiable-build:latest")]
        solana_image: String,

        /// Rust image to run `cargo prove build --docker`
        #[arg(long, default_value = "rust:1.80-bookworm")]
        rust_image: String,

        /// Tag passed to `cargo prove build --docker --tag`
        #[arg(long, default_value = "latest")]
        sp1_tag: String,

        /// Custom output dir (default: target/kas-l2)
        #[arg(long)]
        out_dir: Option<PathBuf>,
    },
}

#[derive(Copy, Clone, Eq, PartialEq, ValueEnum)]
enum LenPrefix { U32, U64 }

#[derive(Debug, Deserialize, Clone)]
struct CargoPackage {
    name: String,
    manifest_path: String,
    targets: Vec<CargoTarget>,
}

#[derive(Debug, Deserialize, Clone)]
struct CargoTarget {
    name: String,
    kind: Vec<String>,         // e.g. ["lib"], ["bin"]
    crate_types: Vec<String>,  // e.g. ["cdylib"]
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::BuildProgram {
            name, prefer_anchor, force_generic, skip_prove, verbose, len,
            anchor_tag, solana_image, rust_image, sp1_tag, out_dir
        } => build_program(
            name, prefer_anchor, force_generic, skip_prove, verbose, len,
            &anchor_tag, &solana_image, &rust_image, &sp1_tag, out_dir
        ),
    }
}

fn build_program(
    name_cli: Option<String>,
    prefer_anchor: bool,
    force_generic: bool,
    skip_prove: bool,
    verbose: bool,
    len: LenPrefix,
    anchor_tag: &str,
    solana_image: &str,
    rust_image: &str,
    sp1_tag: &str,
    out_dir_cli: Option<PathBuf>,
) -> Result<()> {
    which("docker").context("docker is required on the host")?;

    // ---- read cargo metadata (for package & targets) ----
    let meta = cargo_metadata_json()?;
    let workspace_root = PathBuf::from(meta["workspace_root"].as_str().unwrap_or("."));
    let (primary_pkg, pkgs) = select_primary_package(&meta)?;

    // derive default name = primary package name (use package name as-is)
    let out_name = name_cli.unwrap_or_else(|| primary_pkg.name.clone());

    // collect expected target names/kinds from primary package
    let targets = &primary_pkg.targets;
    let expected = derive_expected_filenames(targets);

    if verbose {
        eprintln!("[kas] package: {}", primary_pkg.name);
        eprintln!("[kas] expected artifacts (filenames only):");
        for f in &expected { eprintln!("       - {f}"); }
    }

    // ---- dirs & paths ----
    let cwd = env::current_dir()?;
    let out_dir = out_dir_cli.unwrap_or_else(|| PathBuf::from("target/kas-l2"));
    let stage_dir = out_dir.join("_staging").join(&out_name);
    fs::create_dir_all(&stage_dir).with_context(|| format!("creating {}", stage_dir.display()))?;
    fs::create_dir_all(&out_dir).with_context(|| format!("creating {}", out_dir.display()))?;
    let out_path = out_dir.join(format!("{out_name}.kas"));

    // ---- Solana build inside Docker ----
    let is_anchor_repo = workspace_root.join("Anchor.toml").exists() || Path::new("Anchor.toml").exists();
    let use_anchor = if force_generic { false } else if prefer_anchor { true } else { is_anchor_repo };

    if use_anchor {
        vlog(verbose, "Solana: Anchor verifiable build");
        docker_run(&[
            "run","--rm",
            "-v", &format!("{}:/work", cwd.display()),
            "-w", "/work",
            &format!("solanafoundation/anchor:{anchor_tag}"),
            "anchor","build","--verifiable",
        ], verbose)?;
    } else {
        vlog(verbose, "Solana: generic verifiable-build image");
        docker_run(&[
            "run","--rm",
            "-v", &format!("{}:/work", cwd.display()),
            "-w", "/work",
            solana_image,
            "bash","-lc",
            "set -e; if command -v solana-verify >/dev/null 2>&1; then solana-verify build; \
             else echo 'Container missing solana-verify; adjust --solana-image' >&2; exit 1; fi",
        ], verbose)?;
    }

    // ---- Stage Solana artifacts (from exact names) ----
    let mut staged: Vec<PathBuf> = Vec::new();
    let solana_search_roots = [
        PathBuf::from("target/verifiable"),
        PathBuf::from("target/deploy"),
        // Typical cargo target dirs for sbf/sbpf:
        PathBuf::from("target/sbf-solana-solana/release"),
        PathBuf::from("target/sbpf-solana-solana/release"),
    ];
    stage_expected(&expected, &solana_search_roots, &stage_dir, &mut staged, verbose)?;

    // ---- SP1 build inside Docker (optional) ----
    if !skip_prove {
        vlog(verbose, "SP1: cargo prove build --docker");
        let script = format!(
            "set -euo pipefail; \
             apt-get update -qq; apt-get install -y -qq curl git ca-certificates; \
             if ! command -v cargo >/dev/null 2>&1 {{ curl https://sh.rustup.rs -sSf | sh -s -- -y; }}; \
             export PATH=$HOME/.cargo/bin:$PATH; \
             curl -sSL https://sp1up.succinct.xyz | bash; \
             export PATH=$HOME/.cargo/bin:$PATH; \
             cargo prove build --docker --tag {sp1_tag}"
        );
        docker_run(&[
            "run","--rm",
            "-v", &format!("{}:/work", cwd.display()),
            "-w", "/work",
            rust_image,
            "bash","-lc", &script,
        ], verbose)?;

        // Stage SP1 outputs by *bin names* -> <bin>.elf in release dirs
        let bin_names: Vec<String> = targets.iter()
            .filter(|t| t.kind.iter().any(|k| k == "bin"))
            .map(|t| t.name.clone())
            .collect();

        let sp1_search_roots = [
            PathBuf::from("target"),
            PathBuf::from("."), // in case examples put under specific subdirs
        ];
        stage_sp1_elves(&bin_names, &sp1_search_roots, &stage_dir, &mut staged, verbose)?;
    } else {
        vlog(verbose, "SP1: skipped (--skip-prove)");
    }

    if staged.is_empty() {
        bail!("no artifacts found to package in {}", stage_dir.display());
    }

    // ---- Concatenate <len><bytes>… into target/kas-l2/<name>.kas ----
    vlog(verbose, &format!("packing {} file(s) into {}", staged.len(), out_path.display()));
    staged.sort(); // deterministic order
    let f = fs::File::create(&out_path).with_context(|| format!("creating {}", out_path.display()))?;
    let mut w = BufWriter::new(f);
    for p in &staged {
        let bytes = fs::read(p).with_context(|| format!("reading {}", p.display()))?;
        match len {
            LenPrefix::U32 => {
                let n = u32::try_from(bytes.len()).context("artifact too large for u32 length")?;
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
    for p in staged { println!("  - {}", p.display()); }
    Ok(())
}

// ---------- cargo metadata helpers ----------

fn cargo_metadata_json() -> Result<Json> {
    let out = Command::new("cargo")
        .args(["metadata", "--format-version=1", "--no-deps"])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .context("running `cargo metadata`")?;
    if !out.status.success() { bail!("cargo metadata failed"); }
    Ok(serde_json::from_slice(&out.stdout)?)
}

fn select_primary_package(meta: &Json) -> Result<(CargoPackage, Vec<CargoPackage>)> {
    let cwd = env::current_dir()?.canonicalize()?;
    let pkgs_val = meta["packages"].as_array().unwrap_or(&vec![]).clone();
    let mut pkgs: Vec<CargoPackage> = Vec::new();
    for p in pkgs_val {
        let pkg: CargoPackage = serde_json::from_value(p.clone())?;
        pkgs.push(pkg);
    }

    // primary = the package whose manifest is the workspace root's Cargo.toml, or cwd/Cargo.toml
    let root_manifest = PathBuf::from(meta["workspace_root"].as_str().unwrap_or(".")).join("Cargo.toml");
    let mut primary: Option<CargoPackage> = None;

    // 1) exact root match
    for p in &pkgs {
        if Path::new(&p.manifest_path) == root_manifest {
            primary = Some(p.clone());
            break;
        }
    }
    // 2) manifest in current dir
    if primary.is_none() {
        let cwd_manifest = cwd.join("Cargo.toml");
        for p in &pkgs {
            if Path::new(&p.manifest_path) == cwd_manifest {
                primary = Some(p.clone());
                break;
            }
        }
    }
    // 3) fallback: first package
    let primary = primary.unwrap_or_else(|| pkgs.first().expect("no packages found").clone());
    Ok((primary, pkgs))
}

/// Produce the **filenames** we expect from the target kinds.
/// We intentionally include both `<name>.so` and `lib<name>.so` for cdylib/dylib to cover Solana vs. general Rust conventions.
/// Also include `<name>.rlib` for lib/rlib, and `<name>` (bin) in case you want to package host bins too.
fn derive_expected_filenames(targets: &Vec<CargoTarget>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seen = HashSet::new();

    for t in targets {
        let name = t.name.replace('-', "_"); // cargo normalizes lib names
        let has = |s: &str| t.kind.iter().any(|k| k == s) || t.crate_types.iter().any(|c| c == s);

        if has("cdylib") || has("dylib") {
            // Solana often emits <name>.so for program cdylib; generic Rust dylib is lib<name>.so
            for cand in [format!("{name}.so"), format!("lib{name}.so")] {
                if seen.insert(cand.clone()) { out.push(cand); }
            }
        }
        if has("lib") || has("rlib") {
            let f = format!("lib{name}.rlib");
            if seen.insert(f.clone()) { out.push(f); }
        }
        if has("bin") {
            // host binary
            let f = name.clone();
            if seen.insert(f.clone()) { out.push(f); }
            // SP1 ELF built from a bin target typically named <bin>.elf
            let elf = format!("{name}.elf");
            if seen.insert(elf.clone()) { out.push(elf); }
        }
    }
    out
}

// ---------- staging & docker utils ----------

fn stage_expected(
    expected_filenames: &Vec<String>,
    roots: &[PathBuf],
    stage_dir: &Path,
    staged: &mut Vec<PathBuf>,
    verbose: bool,
) -> Result<()> {
    for fname in expected_filenames {
        let mut found = false;
        // try direct file in each root
        for r in roots {
            let p = r.join(fname);
            if p.is_file() {
                let dest = stage_dir.join(p.file_name().unwrap());
                fs::copy(&p, &dest).with_context(|| format!("copy {} -> {}", p.display(), dest.display()))?;
                staged.push(dest);
                vlog(verbose, &format!("staged {}", p.display()));
                found = true;
                break;
            }
        }
        // try globs if not found (particularly for nested target dirs)
        if !found {
            for r in roots {
                let pat = format!("{}/**/{}", r.display(), fname);
                for entry in glob(&pat).with_context(|| format!("bad glob: {pat}"))? {
                    if let Ok(p) = entry {
                        if p.is_file() {
                            let dest = stage_dir.join(p.file_name().unwrap());
                            fs::copy(&p, &dest)
                                .with_context(|| format!("copy {} -> {}", p.display(), dest.display()))?;
                            staged.push(dest);
                            vlog(verbose, &format!("staged {}", p.display()));
                            found = true;
                            break;
                        }
                    }
                }
                if found { break; }
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
        // no declared bins; fall back to "any .elf under release"
        for r in roots {
            let pat = format!("{}/**/release/*.elf", r.display());
            for entry in glob(&pat).with_context(|| format!("bad glob: {pat}"))? {
                if let Ok(p) = entry {
                    if p.is_file() {
                        let dest = stage_dir.join(p.file_name().unwrap());
                        fs::copy(&p, &dest).with_context(|| format!("copy {} -> {}", p.display(), dest.display()))?;
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
            for entry in glob(&pat).with_context(|| format!("bad glob: {pat}"))? {
                if let Ok(p) = entry {
                    if p.is_file() {
                        let dest = stage_dir.join(p.file_name().unwrap());
                        fs::copy(&p, &dest).with_context(|| format!("copy {} -> {}", p.display(), dest.display()))?;
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
    let status = cmd.status().context("failed to spawn docker")?;
    if !status.success() { bail!("docker command failed"); }
    Ok(())
}

fn vlog(verbose: bool, msg: &str) {
    if verbose { eprintln!("[kas-l2] {msg}"); }
}
