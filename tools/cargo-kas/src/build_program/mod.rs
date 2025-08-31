mod args;
mod sbf;
mod sp1;

pub use crate::build_program::args::Args;

use crate::utils::cargo::CargoTarget;
use crate::{utils::cargo, utils::docker, vlog};
use anyhow::{bail, Result};
use glob::glob;
use std::collections::HashSet;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::{env, fs};

pub fn handle(args: Args) -> Result<()> {
    // Track whether we touched Docker so we can fix ownership at the end.
    let mut used_docker = false;

    // ---- workspace / metadata ----
    let meta = cargo::metadata_json()?;
    let workspace_root = PathBuf::from(meta["workspace_root"].as_str().unwrap_or("."))
        .canonicalize()
        .unwrap_or(env::current_dir()?);
    let cwd = env::current_dir()?.canonicalize()?;
    let (primary_pkg, _) = cargo::select_primary_package(&meta)?;
    let rel_cwd = cwd.strip_prefix(&workspace_root).unwrap_or(Path::new(""));
    let rel_cwd_str = rel_cwd.to_string_lossy().to_string();
    let out_name = args.name.unwrap_or_else(|| primary_pkg.name.clone());

    // Expected artifacts
    let (expected_sbf, sp1_bins) = derive_expected_artifacts(&primary_pkg.targets);

    if args.verbose {
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
        if args.reproducible {
            eprintln!("[kas] reproducible mode: forcing dockerized builds for SBF and SP1");
        }
    }

    // ---- dirs ----
    let out_dir = args.out_dir.unwrap_or_else(|| PathBuf::from("target/kas"));
    let stage_dir = out_dir.join("_staging").join(&out_name);
    fs::create_dir_all(&stage_dir)?;
    fs::create_dir_all(&out_dir)?;
    let out_path = out_dir.join(format!("{out_name}.kas"));

    // ---- build SBF (local preferred unless reproducible) ----
    if !args.skip_sbf {
        used_docker |= sbf::build(
            &workspace_root,
            &rel_cwd_str,
            args.reproducible,
            args.verbose,
            &args.solana_image,
        )?;
    } else {
        vlog(args.verbose, "Solana: skipped");
    }

    // ---- stage SBF artifacts ----
    let mut staged: Vec<PathBuf> = Vec::new();
    let sbf_roots = [workspace_root.join("target/sbpf-solana-solana/release")];
    if args.verbose {
        eprintln!("[kas] SBF artifact dir:");
        for r in &sbf_roots {
            eprintln!("   - {}", r.display());
        }
    }
    stage_expected(
        &expected_sbf,
        &sbf_roots,
        &stage_dir,
        &mut staged,
        args.verbose,
    )?;

    // ---- build SP1 (local preferred unless reproducible) ----
    if !args.skip_prove {
        used_docker |= sp1::build(
            &workspace_root,
            &rel_cwd_str,
            args.reproducible,
            args.verbose,
            &args.rust_image,
        )?;
    } else {
        vlog(args.verbose, "SP1: skipped");
    }

    // ---- stage SP1 artifacts (bins only) ----
    let sp1_release_dir =
        workspace_root.join("target/elf-compilation/riscv32im-succinct-zkvm-elf/release");
    if args.verbose {
        eprintln!("[kas] SP1 release dir:\n   - {}", sp1_release_dir.display());
    }
    stage_sp1_bins(
        &sp1_bins,
        &sp1_release_dir,
        &stage_dir,
        &mut staged,
        args.verbose,
    )?;

    if staged.is_empty() {
        bail!("no artifacts found in {}", stage_dir.display());
    }

    // ---- pack artifacts ----
    staged.sort();
    let f = fs::File::create(&out_path)?;
    let mut w = BufWriter::new(f);

    for p in &staged {
        let bytes = fs::read(p)?;
        w.write_all(&u64::try_from(bytes.len()).unwrap().to_le_bytes())?;
        w.write_all(&bytes)?;
    }
    w.flush()?;

    println!("Wrote {} artifacts -> {}", staged.len(), out_path.display());
    for p in staged {
        println!("  - {}", p.display());
    }

    // ---- finalizer: fix ownership if docker touched the tree ----
    if used_docker {
        if let Err(e) = docker::fix_ownership(
            &env::current_dir()?.canonicalize()?,
            &args.rust_image,
            args.verbose,
        ) {
            eprintln!("[kas] warning: failed to fix ownership after docker build: {e}");
        }
    }

    Ok(())
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
            bail!(
                "staging destination exists and is not a file: {}",
                dest.display()
            );
        }
        // Count pre-existing staged artifacts so packaging works on repeat runs.
        staged.push(dest.clone());
        vlog(
            verbose,
            &format!(
                "    -> already staged {}, counting existing",
                dest.display()
            ),
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
        vlog(
            verbose,
            &format!("searching for expected SBF file: {fname}"),
        );

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
    for name in bin_names {
        let candidate = release_dir.join(name);
        vlog(verbose, &format!("SP1 check {}", candidate.display()));
        if candidate.is_file() {
            stage_copy_unique(&candidate, stage_dir, staged, verbose)?;
        } else {
            vlog(
                verbose,
                &format!("  ! SP1 binary not found: {}", candidate.display()),
            );
        }
    }
    Ok(())
}
