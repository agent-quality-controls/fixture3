use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "goldencheck")]
#[command(about = "CLI for golden fixture behavior checks")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

impl Cli {
    pub(crate) fn parse() -> Result<Self, clap::Error> {
        <Self as Parser>::try_parse()
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Check(CheckArgs),
    Diff(DiffArgs),
    Approve(ApproveArgs),
    Status(StatusArgs),
    Init(InitArgs),
}

#[derive(Debug, Parser)]
pub(crate) struct CheckArgs {
    #[arg(long)]
    pub(crate) suite: String,

    #[arg(long, default_value = "goldencheck.yaml")]
    pub(crate) manifest: PathBuf,
}

#[derive(Debug, Parser)]
pub(crate) struct DiffArgs {
    #[arg(long)]
    pub(crate) suite: String,

    #[arg(long, default_value = "goldencheck.yaml")]
    pub(crate) manifest: PathBuf,

    #[arg(long)]
    pub(crate) refresh: bool,
}

#[derive(Debug, Parser)]
pub(crate) struct ApproveArgs {
    #[arg(long)]
    pub(crate) suite: String,

    #[arg(long, default_value = "goldencheck.yaml")]
    pub(crate) manifest: PathBuf,

    #[arg(long)]
    pub(crate) change: Option<PathBuf>,
}

#[derive(Debug, Parser)]
pub(crate) struct StatusArgs {
    #[arg(long)]
    pub(crate) suite: Option<String>,

    #[arg(long, default_value = "goldencheck.yaml")]
    pub(crate) manifest: PathBuf,
}

#[derive(Debug, Parser)]
pub(crate) struct InitArgs {
    #[arg(long, default_value = "goldencheck.yaml")]
    pub(crate) manifest: PathBuf,
}
