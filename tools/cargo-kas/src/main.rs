mod build_program;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::env;

#[derive(Parser)]
#[command(
    bin_name = "cargo-kas",
    version,
    about = "Dockerized builder/packager for kas artifacts"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Build artifacts; package into target/kas/<name>.kas
    BuildProgram(build_program::Args),
}

fn main() -> Result<()> {
    let mut args: Vec<String> = env::args().collect();
    if args.get(1).is_some_and(|s| s == "kas") {
        args.remove(1);
    }

    match Cli::parse_from(args).cmd {
        Cmd::BuildProgram(args) => build_program::handle(args),
    }
}

pub fn log(msg: &str) {
    eprintln!("[kas] {msg}");
}

pub fn vlog(verbose: bool, msg: &str) {
    if verbose {
        eprintln!("[kas] {msg}");
    }
}
