use std::num::NonZeroUsize;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

const TOP_LEVEL_HELP: &str = "\
fixture3 is a fixture-based approval testing CLI.

It runs project commands against fixture files, parses suite command stdout as JSON,
writes canonical pretty JSON, compares that with committed approved output, and
writes received output and diff files for review. Use it for fixture-based approval
testing where behavior is easier to judge from input/output drift than from
agent-maintained unit tests.

Use fixture3 for:
  CLI output, parser output, diagnostics, codegen, migration plans, API examples,
  rule-engine reports, static-analysis findings, and any stable JSON behavior that
  should not change without approval.

Core concepts:
  fixture: an input file or copied input tree given to the project command.
  suite: one approval check. A suite owns fixture globs, command argv, accepted
    exit codes, and storage directories.
  approved output: committed expected behavior at approved.normalized.json.
  received output: latest command output from a check run.
  diff: the review surface between approved and received output.
  feature: a named group of suites with an optional spec path. fixture3 uses
    features only for selection and reporting.
  reducer: a DDMin-based helper that removes fixture files or directories while
    preserving one suite's approved output.
  A suite is one fixture approval check.

Quick start:
  1. Install the binary.
     cargo binstall fixture3
  2. Run `fixture3 init` to create a usable example fixture3.yaml.
  3. Edit the suite fixture glob, command argv, and storage paths.
  4. Put stable inputs under behavior/fixtures/<suite>/.
  5. Run check once to create an empty approved JSON baseline if it is missing.
  6. Run `fixture3 doctor`.
  7. Run `fixture3 explain --suite <suite>`.
  8. Run `fixture3 check --suite <suite>`.
  9. Review .fixture3/<suite>/diff.txt when output differs.
  10. Run `fixture3 approve --suite <suite> --comment <text>` for intentional drift.

Manifest schema:
  version: 1
  features:
    <feature>:
      spec: \"docs/features/<feature>.md\"
      suites:
        - \"<suite>\"
  suites:
    <suite>:
      tags:
        - \"<tag>\"
      fixtures:
        - \"behavior/fixtures/<suite>/*/input.json\"
      command:
        argv:
          - \"program\"
          - \"{fixtures}\"
        ok_exit_codes:
          - 0
      storage:
        approved_dir: \"behavior/approved/<suite>\"
        received_dir: \".fixture3/<suite>\"
        diff_dir: \".fixture3/<suite>\"

Manifest rules:
  version must be 1.
  suites is required.
  features is optional.
  command.argv is required and must name the executable and arguments.
  command.ok_exit_codes lists acceptable project command exit codes.
  storage paths are per suite and should not overlap.

Command argv rules:
  `{fixtures}` is replaced with every discovered fixture path.
  If an arg is exactly `{fixtures}`, each fixture becomes a separate argv item.
  If `{fixtures}` appears inside a larger arg, fixture paths are joined with spaces.

Output rules:
  The suite command must write JSON to stdout.
  fixture3 parses suite command stdout as JSON and writes canonical pretty JSON before comparison.
  Formatting-only JSON changes do not matter.
  stderr is not compared; command failure is reported separately.

Files written by check:
  approved_dir/approved.normalized.json is the committed approved output.
  If it is missing, check creates it as empty JSON: {}.
  approved_dir/approved.meta.json records approval metadata.
  received_dir/received.raw.json stores command stdout from the latest check.
  received_dir/received.normalized.json stores normalized output.
  received_dir/received.meta.json stores run metadata.
  diff_dir/diff.txt is the human-readable diff.
  diff_dir/diff.json is machine-readable diff state.

Feature pipeline:
  Fixtures are stable inputs.
  approved_dir stores reviewed behavior. Missing approved output starts as {}.
  received_dir stores the latest command output.
  diff_dir stores the review surface.
  tags select loose groups of suites, such as parser or cli.
  features select intentional behavior slices, such as imports or migrations.

Commands:
  init
    Writes a starter fixture3.yaml.
    Parameters: --manifest <path>, default fixture3.yaml.

  new suite <name>
    Creates sample fixture and approved-output files and prints a manifest block.
    Parameters: --manifest <path>, --fixture <file>, --command <program>.

  doctor
    Validates fixture3.yaml without running project behavior.
    Parameters: --manifest <path>, --json.
    Use before check when setup may be wrong.

  explain
    Shows resolved suite config without running project behavior.
    Parameters: --suite <suite>, --manifest <path>, --json.
    Use when fixture discovery or storage paths are unclear.

  check
    Runs suites and compares received output with approved output.
    Select exactly one target: --suite <suite>, --all, --tag <tag>, or --feature <feature>.
    Other parameters: --manifest <path>, --json.
    Use for normal behavior verification.

  diff
    Shows the latest stored diff.
    Parameters: --suite <suite>, --manifest <path>, --refresh, --json.
    Use --refresh to rerun check before showing diff.

  approve
    Promotes received output to approved output.
    Parameters: --suite <suite>, --manifest <path>, --comment <text>.
    Use only after the behavior change has been reviewed.

  status
    Shows approved, received, and diff file state.
    Target is optional: --suite <suite>, --all, --tag <tag>, or --feature <feature>.
    Other parameters: --manifest <path>, --json.

  reduce
    Minimizes copied fixture trees while preserving one suite's approved output.
    Required parameters: --suite <suite>, --fixture-root <path>, --work-dir <path>.
    Other parameters: --manifest <path>, --reducers <list>, --max-oracle-calls <count>.
    Default reducers are dirs,files. Default budgets are dirs=200 and files=100.
    Use this after copying a real project into a fixture and before approving the fixture.

Workflow:
  `fixture3 check --all` runs every suite and returns the highest-severity status.
  `fixture3 check --tag <tag>` runs every suite with that tag.
  `fixture3 check --json` writes suite results as structured JSON.
  `fixture3 reduce` preserves the selected suite's approved output.
  `fixture3 reduce` never edits `--fixture-root` directly.
  `fixture3 explain --suite <suite>` shows the resolved suite configuration.
  `fixture3 doctor` validates manifest paths, fixtures, features, and storage.
  `fixture3 diff --suite <suite> --refresh` reruns check before showing the diff.

Approve:
  `--comment <text>` is optional approval metadata.
  fixture3 records the string in approved.meta.json; it does not interpret it.

Exit codes:
  0  received output matches approved output
  1  received output differs from approved output
  2  tool, manifest, command, JSON, or runtime error
";

const CHECK_HELP: &str = "\
Run one suite or every suite from fixture3.yaml.

check discovers fixtures, runs each suite command, parses stdout as JSON, writes
canonical pretty JSON under `.fixture3/<suite>`, compares received output with
`approved.normalized.json`, and writes diff files.
If approved output is missing, check creates `approved.normalized.json` as `{}` first.

Use `--suite <name>`, `--all`, `--tag <tag>`, or `--feature <feature>`.
Exit code is 2 if any suite errors. Exit code is 1 if any suite differs and no suite errors.
`--json` writes one record per selected suite with status, exit_code, fixture count,
received path, diff path, and error text when a suite fails.

Use this before reviewing behavior changes.
";

const DIFF_HELP: &str = "\
Show the latest stored diff for one suite.

Without `--refresh`, diff reads `.fixture3/<suite>/diff.txt` and does not rerun the
project command. With `--refresh`, it first runs the same behavior as `check`, then
prints the new diff.

Use `--json` when an agent needs the diff status and text without parsing terminal
formatting.
";

const APPROVE_HELP: &str = "\
Publish the last received output as approved output.

approve copies `.fixture3/<suite>/received.normalized.json` to
`behavior/approved/<suite>/approved.normalized.json` and writes approved metadata.
Use `--comment <text>` when the approval needs a human-readable note.
";

const STATUS_HELP: &str = "\
Show one suite or every suite from fixture3.yaml.

Use `--suite <name>`, `--all`, `--tag <tag>`, or `--feature <feature>`.
Omit all target flags to list every suite.
Use `--json` when an agent needs approved, received, and diff booleans per suite.
";

const REDUCE_HELP: &str = "\
Minimize copied fixture trees.

reduce is for copied-project fixtures that are too large.

It uses DDMin to remove fixture directory subtrees and file candidates while the
selected suite still matches its committed approved output. The behavior contract is
the selected suite's approved.normalized.json. If output changes, that candidate set
is rejected. reduce never edits --fixture-root directly.
It preserves the selected suite's approved output.

Use reduce when:
  a fixture was copied from a real project and contains irrelevant files
  an agent needs proof that fixture content is removable under one behavior contract
  the review surface is too large and needs a smaller fixture tree

Do not use reduce when:
  the approved output is missing or not trusted
  the suite output is too weak to prove the behavior you care about
  the fixture root is the real project source tree instead of a disposable fixture copy

Required inputs:
  --suite <suite> names one suite from fixture3.yaml.
  --manifest <path> points to the fixture3 manifest. Default: fixture3.yaml.
  --fixture-root <path> is the copied fixture tree to reduce.
  --work-dir <path> is scratch space for trial trees and reports.

Optional inputs:
  --reducers <list> defaults to dirs,files. Allowed reducers are dirs and files.
  --max-oracle-calls <count> overrides default budgets with one shared total cap.

Reducers:
  dirs removes directory subtrees top-down.
  files removes individual file candidates.
  reducers run left to right.
  default reducer budgets are dirs=200 and files=100.
  with default reducers, reduce spends up to 200 calls on dirs, then 100 on files.
  with --max-oracle-calls, all reducers share that explicit total cap.
  --max-oracle-calls is the shared oracle-call budget override.
  unknown reducer names are errors.
  duplicate reducer names are errors.

Directory reducer:
  Collects every directory under --fixture-root that contains included files.
  Excludes generated directories such as .git, target, node_modules, dist,
  .fixture3, and .goldencheck.
  Rejects symlinks instead of following them.
  Runs top-down by depth.
  Accepts a removed directory only when the suite output still matches.
  Prunes descendants after a parent directory is accepted.

File reducer:
  Starts from the state left by previous reducers.
  Runs DDMin over remaining files.
  Accepts removed files only when the suite output still matches.

Manifest requirements:
  The suite must exist in fixture3.yaml.
  The suite command must write JSON to stdout.
  The suite storage.approved_dir must contain approved.normalized.json.
  The command should observe the behavior being protected.
  Weak suite output proves only weak removability.

Example manifest shape:
  version: 1
  suites:
    my-suite:
      fixtures:
        - \"behavior/fixtures/my-suite/project/**/*\"
      command:
        argv:
          - \"my-program\"
          - \"{fixtures}\"
        ok_exit_codes:
          - 0
      storage:
        approved_dir: \"behavior/approved/my-suite\"
        received_dir: \".fixture3/my-suite\"
        diff_dir: \".fixture3/my-suite\"

Examples:
  fixture3 reduce --suite my-suite --fixture-root behavior/fixtures/my-suite/project --work-dir .fixture3/reduce/my-suite
  fixture3 reduce --suite my-suite --fixture-root behavior/fixtures/my-suite/project --work-dir .fixture3/reduce/my-suite --reducers dirs
  fixture3 reduce --suite my-suite --fixture-root behavior/fixtures/my-suite/project --work-dir .fixture3/reduce/my-suite --reducers files
  fixture3 reduce --suite my-suite --fixture-root behavior/fixtures/my-suite/project --work-dir .fixture3/reduce/my-suite --max-oracle-calls 50

Outputs:
  JSON is written to stdout.
  The same JSON is written to <work-dir>/reduce-report.json.
  Removed file paths are written to <work-dir>/removed-files.txt.
  Remaining file paths are written to <work-dir>/remaining-files.txt.
  Best-so-far report is written to <work-dir>/best/reduce-report.json.
  The current oracle trial uses <work-dir>/trial-current/.
  trial-current is scratch space and may hold the last failed trial.
  The final accepted state is reduce-report.json, not trial-current.

Report fields:
  reducers lists the reducer sequence.
  phases lists per-reducer candidate counts, remaining counts, and guarantee.
  candidate_count is original file count.
  directory_candidate_count is original directory candidate count.
  remaining_count and removed_count are file counts.
  removed_directories lists accepted removed directory subtrees.
  guarantee is complete only when every requested reducer completed its budgeted run.

Guarantees:
  complete means DDMin finished for the requested reducers and budgets.
  incomplete:max-oracle-calls-reached means the active budget stopped reduction.
  incomplete:baseline-not-interesting means the original fixture root did not match approved output.

Exit codes:
  0  reducer completed and produced a report
  2  manifest, fixture-root, work-dir, trial tree, or oracle execution failed
";

const INIT_HELP: &str = "\
Write an example fixture3.yaml manifest.

The generated manifest is a starting point. Replace the fixture glob, command argv,
accepted exit codes, and storage paths with the behavior contract for your project.
";

const EXPLAIN_HELP: &str = "\
Show resolved suite configuration without running project behavior.

explain prints suite tags, feature membership, fixture globs, resolved fixture count,
command argv, storage paths, and current approved/received/diff file state.
It does not run the project command. Use it to debug what fixture3 will run before
checking behavior.
";

const DOCTOR_HELP: &str = "\
Validate fixture3.yaml without running project behavior.

doctor checks feature suite references, fixture globs, command argv, exit-code lists,
and storage path collisions.
It does not run project commands or compare behavior. Exit 0 means the manifest shape
is usable. Exit 2 means setup needs repair.
";

const NEW_HELP: &str = "\
Create fixture approval scaffolding.

new suite creates a sample fixture input, an initial approved output file, and prints
the manifest block to add under `suites:`.
It does not edit fixture3.yaml. The project keeps ownership of manifest formatting,
feature grouping, and review policy.
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
    #[command(about = "Minimize copied fixture trees")]
    #[command(long_about = REDUCE_HELP)]
    Reduce(ReduceArgs),
    #[command(about = "Create an example fixture3.yaml manifest")]
    #[command(long_about = INIT_HELP)]
    Init(InitArgs),
    #[command(about = "Show resolved suite configuration")]
    #[command(long_about = EXPLAIN_HELP)]
    Explain(ExplainArgs),
    #[command(about = "Validate fixture3.yaml without running project behavior")]
    #[command(long_about = DOCTOR_HELP)]
    Doctor(DoctorArgs),
    #[command(about = "Create fixture approval scaffolding")]
    #[command(long_about = NEW_HELP)]
    New(NewArgs),
}

#[derive(Debug, Parser)]
#[command(group(
    clap::ArgGroup::new("target")
        .required(true)
        .args(["suite", "all", "tag", "feature"])
))]
pub(crate) struct CheckArgs {
    #[arg(long, help = "Suite name from fixture3.yaml")]
    pub(crate) suite: Option<String>,

    #[arg(long, help = "Run every suite in fixture3.yaml")]
    pub(crate) all: bool,

    #[arg(long, help = "Run suites with this tag")]
    pub(crate) tag: Option<String>,

    #[arg(long, help = "Run suites listed under this feature")]
    pub(crate) feature: Option<String>,

    #[arg(long, default_value = "fixture3.yaml", help = "Manifest path")]
    pub(crate) manifest: PathBuf,

    #[arg(long, help = "Write machine-readable JSON output")]
    pub(crate) json: bool,
}

#[derive(Debug, Parser)]
pub(crate) struct DiffArgs {
    #[arg(long, help = "Suite name from fixture3.yaml")]
    pub(crate) suite: String,

    #[arg(long, default_value = "fixture3.yaml", help = "Manifest path")]
    pub(crate) manifest: PathBuf,

    #[arg(long, help = "Run check before printing the diff")]
    pub(crate) refresh: bool,

    #[arg(long, help = "Write machine-readable JSON output")]
    pub(crate) json: bool,
}

#[derive(Debug, Parser)]
pub(crate) struct ApproveArgs {
    #[arg(long, help = "Suite name from fixture3.yaml")]
    pub(crate) suite: String,

    #[arg(long, default_value = "fixture3.yaml", help = "Manifest path")]
    pub(crate) manifest: PathBuf,

    #[arg(long, help = "Optional approval comment recorded in approved metadata")]
    pub(crate) comment: Option<String>,
}

#[derive(Debug, Parser)]
#[command(group(
    clap::ArgGroup::new("target")
        .required(false)
        .args(["suite", "all", "tag", "feature"])
))]
pub(crate) struct StatusArgs {
    #[arg(long, help = "Suite name from fixture3.yaml")]
    pub(crate) suite: Option<String>,

    #[arg(long, help = "Show every suite in fixture3.yaml")]
    pub(crate) all: bool,

    #[arg(long, help = "Show suites with this tag")]
    pub(crate) tag: Option<String>,

    #[arg(long, help = "Show suites listed under this feature")]
    pub(crate) feature: Option<String>,

    #[arg(long, default_value = "fixture3.yaml", help = "Manifest path")]
    pub(crate) manifest: PathBuf,

    #[arg(long, help = "Write machine-readable JSON output")]
    pub(crate) json: bool,
}

#[derive(Debug, Parser)]
pub(crate) struct ReduceArgs {
    #[arg(long, help = "Suite name from fixture3.yaml")]
    pub(crate) suite: String,

    #[arg(long, default_value = "fixture3.yaml", help = "Manifest path")]
    pub(crate) manifest: PathBuf,

    #[arg(long, help = "Copied fixture tree to reduce")]
    pub(crate) fixture_root: PathBuf,

    #[arg(long, help = "Scratch directory for trial trees and reports")]
    pub(crate) work_dir: PathBuf,

    #[arg(long, default_value = "dirs,files", help = "Comma-separated reducers: dirs,files")]
    pub(crate) reducers: String,

    #[arg(long, help = "Override default reducer budgets with one shared total oracle-call cap")]
    pub(crate) max_oracle_calls: Option<NonZeroUsize>,
}

#[derive(Debug, Parser)]
pub(crate) struct InitArgs {
    #[arg(long, default_value = "fixture3.yaml", help = "Manifest path to create")]
    pub(crate) manifest: PathBuf,
}

#[derive(Debug, Parser)]
pub(crate) struct ExplainArgs {
    #[arg(long, help = "Suite name from fixture3.yaml")]
    pub(crate) suite: String,

    #[arg(long, default_value = "fixture3.yaml", help = "Manifest path")]
    pub(crate) manifest: PathBuf,

    #[arg(long, help = "Write machine-readable JSON output")]
    pub(crate) json: bool,
}

#[derive(Debug, Parser)]
pub(crate) struct DoctorArgs {
    #[arg(long, default_value = "fixture3.yaml", help = "Manifest path")]
    pub(crate) manifest: PathBuf,

    #[arg(long, help = "Write machine-readable JSON output")]
    pub(crate) json: bool,
}

#[derive(Debug, Parser)]
pub(crate) struct NewArgs {
    #[command(subcommand)]
    pub(crate) command: NewCommands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum NewCommands {
    #[command(about = "Create one fixture approval suite")]
    Suite(NewSuiteArgs),
}

#[derive(Debug, Parser)]
pub(crate) struct NewSuiteArgs {
    pub(crate) name: String,

    #[arg(long, default_value = "fixture3.yaml", help = "Manifest path used for root resolution")]
    pub(crate) manifest: PathBuf,

    #[arg(long, default_value = "input.json", help = "Sample fixture file name")]
    pub(crate) fixture: String,

    #[arg(long, default_value = "cat", help = "Command program for the manifest block")]
    pub(crate) command: String,
}
