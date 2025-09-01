mod args;
mod builder;

pub use args::Args;

pub fn cmd(args: Args) -> anyhow::Result<()> {
    builder::Builder::try_from(args)?
        .print_verbose()?
        .create_target_directories()?
        .build_sbf_artifacts()?
        .stage_sbf_artifacts()?
        .build_sp1_artifacts()?
        .stage_sp1_artifacts()?
        .pack_artifacts()?
        .finalize()
}
