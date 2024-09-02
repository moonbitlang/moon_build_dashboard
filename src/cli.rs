use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
pub struct MoonBuildDashBoardCli {
    #[clap(subcommand)]
    pub subcommand: MoonBuildDashBoardSubcommands,
}

#[derive(Debug, clap::Parser)]
pub enum MoonBuildDashBoardSubcommands {
    Stat(StatSubcommand),
    Transform(TransformSubcommand),
}

#[derive(Debug, clap::Parser)]
pub struct StatSubcommand {
    #[clap(long)]
    pub repo_url: Option<String>,
    #[clap(long)]
    pub file: Option<PathBuf>,
}

#[derive(Debug, clap::Parser)]
pub struct TransformSubcommand {
    #[arg(short, long)]
    pub path: String,
}
