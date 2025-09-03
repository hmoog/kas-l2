use crate::utils::cargo::CargoTarget;
use crate::utils::{cargo, docker};
use crate::vlog;
use anyhow::{bail, Context};
use clap::Args as ClapArgs;
use glob::glob;
use std::collections::HashSet;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{env, fs};
use which::which;

#[derive(ClapArgs, Debug)]
pub struct Args {
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub skip_sbf: bool,
    #[arg(long)]
    pub skip_sp1: bool,
    #[arg(long)]
    pub reproducible: bool,
    #[arg(long)]
    pub verbose: bool,
    #[arg(long, default_value = "solanafoundation/solana-verifiable-build:2.3.6")]
    pub solana_image: String,
    #[arg(long, default_value = "debian:bookworm-slim")]
    pub rust_image: String,
    #[arg(long)]
    pub out_dir: Option<PathBuf>,
}

pub fn cmd(args: Args) -> anyhow::Result<()> {
    Builder::try_from(args)?
        .print_verbose()?
        .create_target_directories()?
        .build_sbf_artifacts()?
        .stage_sbf_artifacts()?
        .build_sp1_artifacts()?
        .stage_sp1_artifacts()?
        .pack_artifacts()?
        .finalize()
}

struct Builder {
    pub(crate) args: Args,
    pub(crate) workspace_root: PathBuf,
    pub(crate) current_directory: PathBuf,
    pub(crate) primary_pkg: cargo::CargoPackage,
    pub(crate) sbf_targets: Vec<String>,
    pub(crate) sp1_targets: Vec<String>,
    pub(crate) target_dir: PathBuf,
    pub(crate) target_file: PathBuf,
    pub(crate) staging_dir: PathBuf,
    pub(crate) staged_files: Vec<PathBuf>,
    pub(crate) used_docker: bool,
}

impl TryFrom<Args> for Builder {
    type Error = anyhow::Error;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        let cargo_metadata = cargo::metadata_json()?;

        let workspace_root =
            PathBuf::from(cargo_metadata["workspace_root"].as_str().unwrap_or("."))
                .canonicalize()
                .unwrap_or(env::current_dir()?);

        let current_directory = env::current_dir()?
            .canonicalize()?
            .strip_prefix(&workspace_root)
            .unwrap_or(Path::new(""))
            .to_owned();

        let (primary_pkg, _) = cargo::select_primary_package(&cargo_metadata)?;
        let (sbf_targets, sp1_targets) = derive_expected_artifacts(&primary_pkg.targets);

        let target_name = args
            .name
            .clone()
            .unwrap_or_else(|| primary_pkg.name.clone());
        let target_dir = args
            .out_dir
            .clone()
            .unwrap_or_else(|| workspace_root.join("target").join("kas"));
        let target_file = target_dir.join(format!("{target_name}.kas"));
        let staging_dir = target_dir.join("_staging").join(&target_name);

        Ok(Builder {
            args,
            workspace_root,
            current_directory,
            primary_pkg,
            sbf_targets,
            sp1_targets,
            target_dir,
            target_file,
            staging_dir,
            staged_files: Vec::new(),
            used_docker: true,
        })
    }
}

impl Builder {
    pub(crate) fn print_verbose(self) -> anyhow::Result<Self> {
        if self.args.verbose {
            eprintln!("[kas] package: {}", self.primary_pkg.name);
            eprintln!("[kas] workspace_root: {}", self.workspace_root.display());
            eprintln!("[kas] cwd: {}", self.current_directory.display());
            if !self.sbf_targets.is_empty() {
                eprintln!("[kas] expected SBF artifacts:");
                for f in &self.sbf_targets {
                    eprintln!("   - {f}");
                }
            }
            if !self.sp1_targets.is_empty() {
                eprintln!("[kas] expected SP1 bin names:");
                for b in &self.sp1_targets {
                    eprintln!("   - {b}");
                }
            }
            if self.args.reproducible {
                eprintln!("[kas] reproducible mode: forcing dockerized builds for SBF and SP1");
            }
        }

        Ok(self)
    }

    pub(crate) fn create_target_directories(self) -> anyhow::Result<Self> {
        fs::create_dir_all(&self.target_dir)?;
        fs::create_dir_all(&self.staging_dir)?;
        Ok(self)
    }

    pub(crate) fn build_sbf_artifacts(mut self) -> anyhow::Result<Self> {
        if self.args.skip_sbf {
            vlog(true, "sbf-generation: skipped");
            return Ok(self);
        }

        if !self.args.reproducible {
            match which("cargo-build-sbf") {
                Ok(_) => {
                    vlog(self.args.verbose, "Solana: cargo-build-sbf (local)");
                    let status = Command::new("cargo-build-sbf")
                        .args(["--", "--lib"])
                        .stdin(Stdio::null())
                        .status();

                    match status {
                        Ok(s) if s.success() => return Ok(self),
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
            vlog(
                self.args.verbose,
                "Solana: reproducible mode -> using Docker",
            );
        }

        which("docker").context("docker is required for SBF build")?;
        self.used_docker = true;

        let cwd = self.current_directory.to_string_lossy().to_string();
        let build_cmd = if cwd.is_empty() {
            "cargo-build-sbf -- --lib".to_string()
        } else {
            format!("cd {cwd} && cargo-build-sbf -- --lib")
        };

        docker::run(
            &[
                "run",
                "--rm",
                "-v",
                &format!("{}:/work", self.workspace_root.to_string_lossy().to_string()),
                "-w",
                "/work",
                &self.args.solana_image,
                "bash",
                "-lc",
                &format!(
                    "set -euo pipefail; \
                 export PATH=/usr/local/cargo/bin:/root/.local/share/solana/install/active_release/bin:$PATH; \
                 {build_cmd}"
                ),
            ],
            self.args.verbose,
        )?;

        Ok(self)
    }

    pub(crate) fn stage_sbf_artifacts(mut self) -> anyhow::Result<Self> {
        let sbf_roots = [self
            .workspace_root
            .join("target/sbpf-solana-solana/release")];

        if self.args.verbose {
            eprintln!("[kas] SBF artifact dir:");
            for r in &sbf_roots {
                eprintln!("   - {}", r.display());
            }
        }

        stage_expected(
            &self.sbf_targets,
            &sbf_roots,
            &self.staging_dir,
            &mut self.staged_files,
            self.args.verbose,
        )?;

        Ok(self)
    }

    pub(crate) fn build_sp1_artifacts(mut self) -> anyhow::Result<Self> {
        if self.args.skip_sp1 {
            vlog(true, "sbf-generation: skipped");
            return Ok(self);
        }

        if !self.args.reproducible {
            match which("cargo-prove") {
                Ok(_) => {
                    vlog(self.args.verbose, "SP1: cargo prove build (local)");
                    let status = Command::new("cargo")
                        .args(["prove", "build"])
                        .stdin(Stdio::null())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .status();

                    match status {
                        Ok(s) if s.success() => return Ok(self),
                        Ok(s) => bail!(
                            "local `cargo prove build` failed (exit code {:?})",
                            s.code()
                        ),
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
            vlog(self.args.verbose, "SP1: reproducible mode -> using Docker");
        }

        which("docker").context("docker is required for SP1 build")?;
        self.used_docker = true;

        let cwd = self.current_directory.to_string_lossy().to_string();
        let inner = if cwd.is_empty() {
            "cargo prove build".to_string()
        } else {
            format!("cd {cwd} && cargo prove build")
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

        docker::run(
            &[
                "run",
                "--rm",
                "-v",
                &format!("{}:/work", self.workspace_root.display()),
                "-w",
                "/work",
                &self.args.rust_image,
                "bash",
                "-lc",
                &script,
            ],
            self.args.verbose,
        )?;

        Ok(self)
    }

    pub(crate) fn stage_sp1_artifacts(mut self) -> anyhow::Result<Self> {
        let sp1_release_dir = self
            .workspace_root
            .join("target/elf-compilation/riscv32im-succinct-zkvm-elf/release");

        if self.args.verbose {
            eprintln!("[kas] SP1 release dir:\n   - {}", sp1_release_dir.display());
        }

        stage_expected(
            &self.sp1_targets,
            &[sp1_release_dir],
            &self.staging_dir,
            &mut self.staged_files,
            self.args.verbose,
        )?;

        Ok(self)
    }

    pub(crate) fn pack_artifacts(self) -> anyhow::Result<Self> {
        if self.staged_files.is_empty() {
            bail!("no artifacts found in {}", self.staging_dir.display());
        }

        let f = fs::File::create(&self.target_file)?;
        let mut w = BufWriter::new(f);
        for p in &self.staged_files {
            let bytes = fs::read(p)?;
            w.write_all(&u64::try_from(bytes.len()).unwrap().to_le_bytes())?;
            w.write_all(&bytes)?;
        }
        w.flush()?;

        println!(
            "Packed {} artifacts -> {}",
            self.staged_files.len(),
            self.target_file.display()
        );
        for p in &self.staged_files {
            println!("  - {}", p.display());
        }

        Ok(self)
    }

    pub(crate) fn finalize(self) -> anyhow::Result<()> {
        if self.used_docker {
            if let Err(e) = docker::fix_ownership(
                &env::current_dir()?.canonicalize()?,
                &self.args.rust_image,
                self.args.verbose,
            ) {
                eprintln!("[kas] warning: failed to fix ownership after docker build: {e}");
            }
        }

        Ok(())
    }
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
) -> anyhow::Result<()> {
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
) -> anyhow::Result<()> {
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
