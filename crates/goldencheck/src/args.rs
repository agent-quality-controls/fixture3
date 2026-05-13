use std::path::PathBuf;

use clap::{Parser, Subcommand};

const TOP_LEVEL_HELP: &str = "\
goldencheck runs project commands against fixture files and compares normalized output with
committed approved output.

Workflow:
  1. Write goldencheck.yaml with one or more suites.
  2. Put fixture inputs under behavior/fixtures.
  3. Put approved output under behavior/golden/<suite>.
  4. Run `goldencheck check --suite <name>`.
  5. Review `.goldencheck/<suite>/diff.txt`.
  6. Run `goldencheck approve --suite <name> --change <path>` when the change is intentional.

A suite is one golden behavior check. It defines fixture globs, the command to run,
accepted exit codes, optional output normalization, and approved/received/diff storage.

Exit codes:
  0  received output matches approved output
  1  received output differs from approved output
  2  tool, manifest, command, normalizer, or runtime error
";

const CHECK_HELP: &str = "\
Run one suite from goldencheck.yaml.

check discovers fixtures, runs the suite command, optionally runs the normalizer,
normalizes JSON output, writes received files under `.goldencheck/<suite>`, compares
received output with `approved.normalized.json`, and writes diff files.

Use this before reviewing behavior changes.
";

const DIFF_HELP: &str = "\
Show the latest stored diff for one suite.

Without `--refresh`, diff reads `.goldencheck/<suite>/diff.txt` and does not rerun the
project command. With `--refresh`, it first runs the same behavior as `check`, then
prints the new diff.
";

const APPROVE_HELP: &str = "\
Publish the last received output as approved output.

approve copies `.goldencheck/<suite>/received.normalized.json` to
`behavior/golden/<suite>/approved.normalized.json` and writes approved metadata.
If the stored diff says output changed, `--change <path>` is required so the approval
records the reviewed change file.
";

const STATUS_HELP: &str = "\
Report whether approved output, received output, and diff files exist for configured suites.

Use `--suite <name>` to show one suite, or omit it to list every suite in the manifest.
";

const INIT_HELP: &str = "\
Write an example goldencheck.yaml manifest.

The generated manifest is a starting point. Replace the fixture glob, command argv,
accepted exit codes, and storage paths with the behavior contract for your project.
";

#[derive(Debug, Parser)]
#[command(name = "goldencheck")]
#[command(version)]
#[command(about = "Fixture-based golden output checks")]
#[command(long_about = TOP_LEVEL_HELP)]
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
    #[command(about = "Run a suite and compare received output with approved output")]
    #[command(long_about = CHECK_HELP)]
    Check(CheckArgs),
    #[command(about = "Show the latest stored diff, optionally refreshing first")]
    #[command(long_about = DIFF_HELP)]
    Diff(DiffArgs),
    #[command(about = "Promote received output to approved output")]
    #[command(long_about = APPROVE_HELP)]
    Approve(ApproveArgs),
    #[command(about = "Show approved, received, and diff file state")]
    #[command(long_about = STATUS_HELP)]
    Status(StatusArgs),
    #[command(about = "Create an example goldencheck.yaml manifest")]
    #[command(long_about = INIT_HELP)]
    Init(InitArgs),
}

#[derive(Debug, Parser)]
pub(crate) struct CheckArgs {
    #[arg(long, help = "Suite name from goldencheck.yaml")]
    pub(crate) suite: String,

    #[arg(long, default_value = "goldencheck.yaml", help = "Manifest path")]
    pub(crate) manifest: PathBuf,
}

#[derive(Debug, Parser)]
pub(crate) struct DiffArgs {
    #[arg(long, help = "Suite name from goldencheck.yaml")]
    pub(crate) suite: String,

    #[arg(long, default_value = "goldencheck.yaml", help = "Manifest path")]
    pub(crate) manifest: PathBuf,

    #[arg(long, help = "Run check before printing the diff")]
    pub(crate) refresh: bool,
}

#[derive(Debug, Parser)]
pub(crate) struct ApproveArgs {
    #[arg(long, help = "Suite name from goldencheck.yaml")]
    pub(crate) suite: String,

    #[arg(long, default_value = "goldencheck.yaml", help = "Manifest path")]
    pub(crate) manifest: PathBuf,

    #[arg(long, help = "Reviewed change file required when output differs")]
    pub(crate) change: Option<PathBuf>,
}

#[derive(Debug, Parser)]
pub(crate) struct StatusArgs {
    #[arg(long, help = "Suite name from goldencheck.yaml")]
    pub(crate) suite: Option<String>,

    #[arg(long, default_value = "goldencheck.yaml", help = "Manifest path")]
    pub(crate) manifest: PathBuf,
}

#[derive(Debug, Parser)]
pub(crate) struct InitArgs {
    #[arg(long, default_value = "goldencheck.yaml", help = "Manifest path to create")]
    pub(crate) manifest: PathBuf,
}
