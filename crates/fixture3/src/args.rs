use std::path::PathBuf;

use clap::{Parser, Subcommand};

const TOP_LEVEL_HELP: &str = "\
fixture3 runs project commands against fixture files and compares normalized output with
committed approved output.

Use it for fixture-based approval testing where the important output is a file or
stdout value: CLI output, parser output, codegen output, API examples, diagnostics,
migrations, or any stable JSON result that should not change without review.

A suite is one fixture approval check. It defines fixture globs, the command to run,
accepted exit codes, output handling, and approved/received/diff storage.

Quick start:
  1. Run `fixture3 init` to create a usable example fixture3.yaml.
  2. Edit the example suite for your command and fixture paths.
  3. Add approved output at behavior/golden/<suite>/approved.normalized.json.
  4. Run `fixture3 check --suite <suite>`.
  5. Review `.fixture3/<suite>/diff.txt`.
  6. Run `fixture3 approve --suite <suite> --change <path>` for intentional drift.

Manifest schema:
  version: 1
  suites:
    <suite>:
      fixtures:
        - \"behavior/fixtures/<suite>/*/input.json\"
      command:
        argv:
          - \"program\"
          - \"{fixtures}\"
        ok_exit_codes:
          - 0
      output:
        format: \"json\"
        normalizer:
          argv:
            - \"optional-normalizer\"
      storage:
        approved_dir: \"behavior/golden/<suite>\"
        received_dir: \".fixture3/<suite>\"
        diff_dir: \".fixture3/<suite>\"

Command argv:
  `{fixtures}` is replaced with every discovered fixture path.
  If an arg is exactly `{fixtures}`, each fixture becomes a separate argv item.
  If `{fixtures}` appears inside a larger arg, fixture paths are joined with spaces.

Output:
  The only supported `output.format` is `json`.
  `output.normalizer` is optional.
  The command stdout must be JSON after the optional normalizer runs.
  JSON is pretty-printed before comparison so formatting-only changes do not matter.

Files:
  approved_dir/approved.normalized.json is the committed approved output.
  approved_dir/approved.meta.json records approved fixture, manifest, and normalizer hashes.
  received_dir/received.raw.json stores command stdout from the latest check.
  received_dir/received.normalized.json stores normalized output from the latest check.
  received_dir/received.meta.json stores run metadata from the latest check.
  diff_dir/diff.txt is the human-readable diff.
  diff_dir/diff.json is the machine-readable diff status.

Workflow:
  `fixture3 check --suite <suite>` runs one suite and compares approved output.
  `fixture3 check --all` runs every suite and returns the highest-severity status.
  `fixture3 diff --suite <suite>` shows the latest stored diff.
  `fixture3 diff --suite <suite> --refresh` reruns check before showing the diff.
  `fixture3 approve --suite <suite> --change <path>` promotes received output.
  `fixture3 status` lists approved, received, and diff file state.

Approve:
  `--change <path>` is required when output differs.
  The path should point to the reviewed change note, issue, PR, or local change file.
  fixture3 records the string in approved.meta.json; it does not read that file.

Exit codes:
  0  received output matches approved output
  1  received output differs from approved output
  2  tool, manifest, command, normalizer, or runtime error
";

const CHECK_HELP: &str = "\
Run one suite or every suite from fixture3.yaml.

check discovers fixtures, runs each suite command, optionally runs the normalizer,
normalizes JSON output, writes received files under `.fixture3/<suite>`, compares
received output with `approved.normalized.json`, and writes diff files.

Use `--suite <name>` for one suite or `--all` for every suite. Exit code is 2 if any
suite errors. Exit code is 1 if any suite differs and no suite errors.

Use this before reviewing behavior changes.
";

const DIFF_HELP: &str = "\
Show the latest stored diff for one suite.

Without `--refresh`, diff reads `.fixture3/<suite>/diff.txt` and does not rerun the
project command. With `--refresh`, it first runs the same behavior as `check`, then
prints the new diff.
";

const APPROVE_HELP: &str = "\
Publish the last received output as approved output.

approve copies `.fixture3/<suite>/received.normalized.json` to
`behavior/golden/<suite>/approved.normalized.json` and writes approved metadata.
If the stored diff says output changed, `--change <path>` is required so the approval
records the reviewed change file.
";

const STATUS_HELP: &str = "\
Show one suite or every suite from fixture3.yaml.

Use `--suite <name>` to show one suite, `--all` to show every suite, or omit both flags
to list every suite.
";

const INIT_HELP: &str = "\
Write an example fixture3.yaml manifest.

The generated manifest is a starting point. Replace the fixture glob, command argv,
accepted exit codes, and storage paths with the behavior contract for your project.
";

#[derive(Debug, Parser)]
#[command(name = "fixture3")]
#[command(version)]
#[command(about = "Fixture-based approval testing")]
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
    #[command(about = "Create an example fixture3.yaml manifest")]
    #[command(long_about = INIT_HELP)]
    Init(InitArgs),
}

#[derive(Debug, Parser)]
#[command(group(
    clap::ArgGroup::new("target")
        .required(true)
        .args(["suite", "all"])
))]
pub(crate) struct CheckArgs {
    #[arg(long, conflicts_with = "all", help = "Suite name from fixture3.yaml")]
    pub(crate) suite: Option<String>,

    #[arg(long, conflicts_with = "suite", help = "Run every suite in fixture3.yaml")]
    pub(crate) all: bool,

    #[arg(long, default_value = "fixture3.yaml", help = "Manifest path")]
    pub(crate) manifest: PathBuf,
}

#[derive(Debug, Parser)]
pub(crate) struct DiffArgs {
    #[arg(long, help = "Suite name from fixture3.yaml")]
    pub(crate) suite: String,

    #[arg(long, default_value = "fixture3.yaml", help = "Manifest path")]
    pub(crate) manifest: PathBuf,

    #[arg(long, help = "Run check before printing the diff")]
    pub(crate) refresh: bool,
}

#[derive(Debug, Parser)]
pub(crate) struct ApproveArgs {
    #[arg(long, help = "Suite name from fixture3.yaml")]
    pub(crate) suite: String,

    #[arg(long, default_value = "fixture3.yaml", help = "Manifest path")]
    pub(crate) manifest: PathBuf,

    #[arg(long, help = "Reviewed change file required when output differs")]
    pub(crate) change: Option<PathBuf>,
}

#[derive(Debug, Parser)]
pub(crate) struct StatusArgs {
    #[arg(long, conflicts_with = "all", help = "Suite name from fixture3.yaml")]
    pub(crate) suite: Option<String>,

    #[arg(long, conflicts_with = "suite", help = "Show every suite in fixture3.yaml")]
    pub(crate) all: bool,

    #[arg(long, default_value = "fixture3.yaml", help = "Manifest path")]
    pub(crate) manifest: PathBuf,
}

#[derive(Debug, Parser)]
pub(crate) struct InitArgs {
    #[arg(long, default_value = "fixture3.yaml", help = "Manifest path to create")]
    pub(crate) manifest: PathBuf,
}
