use clap::Args as ClapArgs;
use std::path::PathBuf;

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
