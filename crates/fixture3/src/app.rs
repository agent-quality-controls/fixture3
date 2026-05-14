use std::fmt::Write as _;
use std::io::{self, Write as IoWrite};
use std::process::ExitCode;

use clap::Error as ClapError;

use crate::args::{ApproveArgs, CheckArgs, Cli, Commands, DiffArgs, InitArgs, StatusArgs};
use crate::command;
use crate::diff;
use crate::error::AppError;
use crate::fixture;
use crate::manifest;
use crate::metadata;
use crate::normalize;
use crate::storage;

type SuiteNames = Vec<String>;

#[derive(Debug)]
pub(crate) struct AppOutcome {
    pub(crate) exit_code: u8,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

#[must_use]
pub fn run_to_stdio() -> ExitCode {
    let outcome = match Cli::parse() {
        Ok(cli) => run(cli),
        Err(error) => AppOutcome::from_clap_error(&error),
    };

    let stdout_result = io::stdout().write_all(outcome.stdout.as_bytes());
    let stderr_result = io::stderr().write_all(outcome.stderr.as_bytes());

    if stdout_result.is_err() || stderr_result.is_err() {
        return ExitCode::from(2);
    }

    ExitCode::from(outcome.exit_code)
}

impl AppOutcome {
    pub(crate) fn tool_error(message: &str) -> Self {
        Self { exit_code: 2, stdout: String::new(), stderr: format!("{message}\n") }
    }

    fn from_clap_error(error: &ClapError) -> Self {
        let code = u8::try_from(error.exit_code()).map_or(2, |value| value);
        let message = error.to_string();
        if error.use_stderr() {
            Self { exit_code: code, stdout: String::new(), stderr: message }
        } else {
            Self { exit_code: code, stdout: message, stderr: String::new() }
        }
    }
}

pub(crate) fn run(cli: Cli) -> AppOutcome {
    match run_checked(cli) {
        Ok(outcome) => outcome,
        Err(error) => AppOutcome::tool_error(&error.to_string()),
    }
}

fn run_checked(cli: Cli) -> Result<AppOutcome, AppError> {
    match cli.command {
        Commands::Check(args) => check(&args),
        Commands::Diff(args) => diff_command(&args),
        Commands::Approve(args) => approve(&args),
        Commands::Status(args) => status(&args),
        Commands::Init(args) => init(&args),
    }
}

fn check(args: &CheckArgs) -> Result<AppOutcome, AppError> {
    let suite_names = selected_suite_names(args.suite.as_deref(), args.all, &args.manifest)?;
    let mut exit_code = 0;
    let mut stdout = String::new();
    let mut stderr = String::new();

    for suite_name in suite_names {
        match run_check(&suite_name, &args.manifest) {
            Ok(result) => {
                exit_code = exit_code.max(result.report.exit_code());
                stdout.push_str(&result.stdout);
                stderr.push_str(&result.stderr);
            }
            Err(error) => {
                exit_code = 2;
                write!(&mut stderr, "suite: {suite_name}\n{error}\n").map_err(|source| {
                    AppError::Manifest(format!("failed to render error: {source}"))
                })?;
            }
        }
    }

    Ok(AppOutcome { exit_code, stdout, stderr })
}

fn diff_command(args: &DiffArgs) -> Result<AppOutcome, AppError> {
    if args.refresh {
        let result = run_check(&args.suite, &args.manifest)?;
        return Ok(AppOutcome {
            exit_code: result.report.exit_code(),
            stdout: result.diff_text,
            stderr: result.stderr,
        });
    }

    let loaded = load_suite(&args.suite, &args.manifest)?;
    let (report, diff_text) = storage::read_diff(&loaded.suite.storage)?;
    Ok(AppOutcome { exit_code: report.exit_code(), stdout: diff_text, stderr: String::new() })
}

fn approve(args: &ApproveArgs) -> Result<AppOutcome, AppError> {
    let loaded = load_suite(&args.suite, &args.manifest)?;
    let change_path = args.change.as_ref().map(|path| path.to_string_lossy().into_owned());
    storage::approve_received(&loaded.suite.storage, change_path)?;
    Ok(AppOutcome {
        exit_code: 0,
        stdout: format!("suite: {}\nstatus: approved\n", args.suite),
        stderr: String::new(),
    })
}

fn status(args: &StatusArgs) -> Result<AppOutcome, AppError> {
    let manifest = manifest::load(&args.manifest)?;
    let suite_names =
        selected_suite_names_from_manifest(args.suite.as_deref(), args.all, &manifest);
    let mut output = String::new();

    for name in suite_names {
        let suite = manifest
            .suites
            .get(&name)
            .ok_or_else(|| AppError::Manifest(format!("suite not found in manifest: {name}")))?;
        let state = storage::status(&suite.storage);
        write!(
            &mut output,
            "suite: {name}\napproved: {}\nreceived: {}\ndiff: {}\n",
            yes_no(state.approved_exists),
            yes_no(state.received_exists),
            yes_no(state.diff_exists)
        )
        .map_err(|source| AppError::Manifest(format!("failed to render status: {source}")))?;
    }

    if output.is_empty() {
        return Err(AppError::Manifest("suite not found in manifest".to_owned()));
    }

    Ok(AppOutcome { exit_code: 0, stdout: output, stderr: String::new() })
}

fn init(args: &InitArgs) -> Result<AppOutcome, AppError> {
    storage::init_manifest(&args.manifest)?;
    Ok(AppOutcome {
        exit_code: 0,
        stdout: format!("manifest: {}\nstatus: initialized\n", args.manifest.display()),
        stderr: String::new(),
    })
}

#[derive(Debug)]
struct LoadedSuite {
    suite: crate::manifest::SuiteConfig,
}

#[derive(Debug)]
struct CheckResult {
    report: diff::DiffReport,
    stdout: String,
    stderr: String,
    diff_text: String,
}

fn selected_suite_names(
    suite_name: Option<&str>,
    all: bool,
    manifest_path: &std::path::Path,
) -> Result<SuiteNames, AppError> {
    let manifest = manifest::load(manifest_path)?;
    Ok(selected_suite_names_from_manifest(suite_name, all, &manifest))
}

fn selected_suite_names_from_manifest(
    suite_name: Option<&str>,
    all: bool,
    manifest: &manifest::Manifest,
) -> SuiteNames {
    if all {
        return manifest.suites.keys().cloned().collect();
    }

    suite_name
        .map_or_else(|| manifest.suites.keys().cloned().collect(), |name| vec![name.to_owned()])
}

fn run_check(suite_name: &str, manifest_path: &std::path::Path) -> Result<CheckResult, AppError> {
    let manifest = manifest::load(manifest_path)?;
    let suite = manifest
        .suites
        .get(suite_name)
        .ok_or_else(|| AppError::Manifest(format!("suite not found in manifest: {suite_name}")))?;
    let fixtures = fixture::discover(suite)?;
    let command_output = command::run_fixture_command(&suite.command, &fixtures)?;
    let normalized = normalize::normalize(&command_output.stdout, &suite.output)?;
    let metadata = metadata::build(suite_name, suite, manifest_path, &fixtures)?;
    let stored =
        storage::write_received(&suite.storage, &command_output.stdout, &normalized, &metadata)?;
    let (report, diff_text) = diff::compare(&stored.approved, &normalized);
    storage::write_diff(&suite.storage, &report, &diff_text)?;

    let stderr = String::from_utf8(command_output.stderr)
        .map_err(|source| AppError::Utf8 { context: "command stderr".to_owned(), source })?;
    let stdout = format!(
        "suite: {suite_name}\nreceived_run_id: {}\nreceived_run_commit: {}\nfixtures: {}\nstatus: {}\nreceived: {}\ndiff: {}\n",
        metadata.run_id,
        metadata.run_commit,
        fixtures.len(),
        report.status.as_str(),
        stored.received_path.display(),
        stored.diff_path.display()
    );

    Ok(CheckResult { report, stdout, stderr, diff_text })
}

fn load_suite(suite_name: &str, manifest_path: &std::path::Path) -> Result<LoadedSuite, AppError> {
    let mut manifest = manifest::load(manifest_path)?;
    let suite = manifest
        .suites
        .remove(suite_name)
        .ok_or_else(|| AppError::Manifest(format!("suite not found in manifest: {suite_name}")))?;
    Ok(LoadedSuite { suite })
}

const fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
