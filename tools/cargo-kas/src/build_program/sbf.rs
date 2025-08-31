use crate::utils::docker;
use crate::vlog;
use anyhow::{bail, Context};
use std::path::Path;
use std::process::{Command, Stdio};
use which::which;

pub fn build(
    workspace_root: &Path,
    rel_cwd: &str,
    reproducible: bool,
    verbose: bool,
    solana_image: &str,
) -> anyhow::Result<bool> {
    let mut used_docker = false;

    if !reproducible {
        match which("cargo-build-sbf") {
            Ok(_) => {
                vlog(verbose, "Solana: cargo-build-sbf (local)");
                let status = Command::new("cargo-build-sbf")
                    .args(["--", "--lib"])
                    .stdin(Stdio::null())
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

    docker::run(
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
