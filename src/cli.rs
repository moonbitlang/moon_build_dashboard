use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
pub struct MoonBuildDashBoardCli {
    #[clap(subcommand)]
    pub subcommand: MoonBuildDashBoardSubcommands,
}

#[derive(Debug, clap::Parser)]
pub enum MoonBuildDashBoardSubcommands {
    Stat(StatSubcommand),
}

#[derive(Debug, clap::Parser)]
pub struct StatSubcommand {
    #[clap(long)]
    pub repo_url: Option<String>,
    #[clap(long)]
    pub file: Option<PathBuf>,
}
