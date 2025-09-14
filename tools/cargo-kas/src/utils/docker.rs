use std::{
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};
use which::which;

use crate::vlog;

pub fn run(args: &[&str], verbose: bool) -> Result<()> {
    vlog(verbose, &format!("docker {}", args.join(" ")));
    if !Command::new("docker")
        .args(args)
        .stdin(Stdio::null())
        .status()?
        .success()
    {
        bail!("docker command failed");
    }
    Ok(())
}

pub fn fix_ownership(workspace_root: &Path, image: &str, verbose: bool) -> Result<()> {
    which("docker").context("docker is required to fix ownership after docker builds")?;
    let script = "set -eu; OWNER=$(stat -c '%u:%g' /work 2>/dev/null || echo 0:0); chown -R \"$OWNER\" /work";
    run(
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
