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
    rust_image: &str,
) -> anyhow::Result<bool> {
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

    docker::run(
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
