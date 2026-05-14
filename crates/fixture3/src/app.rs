use std::fmt::Write as _;
use std::io::{self, Write as IoWrite};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Error as ClapError;
use serde::Serialize;

use crate::error::AppError;
use crate::{
    args, command, diff, doctor, fixture, manifest, metadata, normalize, scaffold, selection,
    storage,
};

#[derive(Debug)]
pub(crate) struct AppOutcome {
    pub(crate) exit_code: u8,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

#[must_use]
pub fn run_to_stdio() -> ExitCode {
    let outcome = match args::Cli::parse() {
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

pub(crate) fn run(cli: args::Cli) -> AppOutcome {
    match run_checked(cli) {
        Ok(outcome) => outcome,
        Err(error) => AppOutcome::tool_error(&error.to_string()),
    }
}

fn run_checked(cli: args::Cli) -> Result<AppOutcome, AppError> {
    match cli.command {
        args::Commands::Check(args) => check(&args),
        args::Commands::Diff(args) => diff_command(&args),
        args::Commands::Approve(args) => approve(&args),
        args::Commands::Status(args) => status(&args),
        args::Commands::Init(args) => init(&args),
        args::Commands::Explain(args) => explain(&args),
        args::Commands::Doctor(args) => doctor_command(&args),
        args::Commands::New(args) => new_command(&args),
    }
}

fn check(args: &args::CheckArgs) -> Result<AppOutcome, AppError> {
    let manifest = manifest::load(&args.manifest)?;
    let suite_names = selection::suite_names(
        &manifest,
        selection::Selector {
            suite: args.suite.as_deref(),
            all: args.all,
            tag: args.tag.as_deref(),
            feature: args.feature.as_deref(),
            default_all: false,
        },
    )?;
    let mut exit_code = 0;
    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut records = Vec::new();

    for suite_name in suite_names {
        match run_check(&suite_name, &args.manifest) {
            Ok(result) => {
                exit_code = exit_code.max(result.report.exit_code());
                if args.json {
                    records.push(CheckRecord::from_result(&suite_name, &result));
                } else {
                    stdout.push_str(&result.stdout);
                }
                stderr.push_str(&result.stderr);
            }
            Err(error) => {
                exit_code = 2;
                if args.json {
                    records.push(CheckRecord::from_error(&suite_name, &error));
                } else {
                    render_check_error(&mut stderr, &suite_name, &error)?;
                }
            }
        }
    }

    if args.json {
        stdout = json(&CheckJson { exit_code, suites: records })?;
    }

    Ok(AppOutcome { exit_code, stdout, stderr })
}

fn diff_command(args: &args::DiffArgs) -> Result<AppOutcome, AppError> {
    if args.refresh {
        let result = run_check(&args.suite, &args.manifest)?;
        if args.json {
            return Ok(AppOutcome {
                exit_code: result.report.exit_code(),
                stdout: json(&DiffJson { report: &result.report, text: &result.diff_text })?,
                stderr: result.stderr,
            });
        }
        return Ok(AppOutcome {
            exit_code: result.report.exit_code(),
            stdout: result.diff_text,
            stderr: result.stderr,
        });
    }

    let loaded = load_suite(&args.suite, &args.manifest)?;
    let (report, diff_text) = storage::read_diff(&loaded.suite.storage)?;
    let stdout =
        if args.json { json(&DiffJson { report: &report, text: &diff_text })? } else { diff_text };
    Ok(AppOutcome { exit_code: report.exit_code(), stdout, stderr: String::new() })
}

fn approve(args: &args::ApproveArgs) -> Result<AppOutcome, AppError> {
    let loaded = load_suite(&args.suite, &args.manifest)?;
    let change_path = args.change.as_ref().map(|path| path.to_string_lossy().into_owned());
    storage::approve_received(&loaded.suite.storage, change_path)?;
    Ok(AppOutcome {
        exit_code: 0,
        stdout: format!("suite: {}\nstatus: approved\n", args.suite),
        stderr: String::new(),
    })
}

fn status(args: &args::StatusArgs) -> Result<AppOutcome, AppError> {
    let manifest = manifest::load(&args.manifest)?;
    let suite_names = selection::suite_names(
        &manifest,
        selection::Selector {
            suite: args.suite.as_deref(),
            all: args.all,
            tag: args.tag.as_deref(),
            feature: args.feature.as_deref(),
            default_all: true,
        },
    )?;
    let mut output = String::new();
    let mut records = Vec::new();

    for name in suite_names {
        let suite = manifest
            .suites
            .get(&name)
            .ok_or_else(|| AppError::Manifest(format!("suite not found in manifest: {name}")))?;
        let state = storage::status(&suite.storage);
        if args.json {
            records.push(StatusRecord {
                suite: name,
                approved: state.approved_exists,
                received: state.received_exists,
                diff: state.diff_exists,
            });
        } else {
            write!(
                &mut output,
                "suite: {name}\napproved: {}\nreceived: {}\ndiff: {}\n",
                yes_no(state.approved_exists),
                yes_no(state.received_exists),
                yes_no(state.diff_exists)
            )
            .map_err(|source| AppError::Manifest(format!("failed to render status: {source}")))?;
        }
    }

    if args.json {
        output = json(&StatusJson { suites: records })?;
    } else if output.is_empty() {
        return Err(AppError::Manifest("suite not found in manifest".to_owned()));
    }

    Ok(AppOutcome { exit_code: 0, stdout: output, stderr: String::new() })
}

fn init(args: &args::InitArgs) -> Result<AppOutcome, AppError> {
    storage::init_manifest(&args.manifest)?;
    Ok(AppOutcome {
        exit_code: 0,
        stdout: format!("manifest: {}\nstatus: initialized\n", args.manifest.display()),
        stderr: String::new(),
    })
}

fn explain(args: &args::ExplainArgs) -> Result<AppOutcome, AppError> {
    let manifest = manifest::load(&args.manifest)?;
    let suite = manifest.suites.get(&args.suite).ok_or_else(|| {
        AppError::Manifest(format!("suite not found in manifest: {}", args.suite))
    })?;
    let fixtures = fixture::discover(suite)?;
    let features = manifest
        .features
        .iter()
        .filter(|(_, feature)| feature.suites.iter().any(|name| name == &args.suite))
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    let state = storage::status(&suite.storage);
    let record = ExplainRecord {
        suite: args.suite.clone(),
        tags: suite.tags.clone(),
        features,
        fixture_globs: suite.fixtures.clone(),
        fixture_count: fixtures.len(),
        fixtures: fixtures.iter().map(|path| path.to_string_lossy().into_owned()).collect(),
        command_argv: suite.command.argv.clone(),
        ok_exit_codes: suite.command.ok_exit_codes.clone(),
        approved_dir: suite.storage.approved.to_string_lossy().into_owned(),
        received_dir: suite.storage.received.to_string_lossy().into_owned(),
        diff_dir: suite.storage.diff.to_string_lossy().into_owned(),
        approved_exists: state.approved_exists,
        received_exists: state.received_exists,
        diff_exists: state.diff_exists,
    };

    let stdout = if args.json { json(&record)? } else { record.render_text()? };
    Ok(AppOutcome { exit_code: 0, stdout, stderr: String::new() })
}

fn doctor_command(args: &args::DoctorArgs) -> Result<AppOutcome, AppError> {
    let manifest = manifest::load(&args.manifest)?;
    let findings = doctor::inspect(&manifest);
    let exit_code = if findings.is_empty() { 0 } else { 2 };
    let stdout = if args.json {
        json(&DoctorJson { findings })?
    } else if findings.is_empty() {
        "status: ok\n".to_owned()
    } else {
        let mut output = String::new();
        for finding in findings {
            writeln!(&mut output, "{}: {}", finding.code, finding.message).map_err(|source| {
                AppError::Manifest(format!("failed to render doctor output: {source}"))
            })?;
        }
        output
    };
    Ok(AppOutcome { exit_code, stdout, stderr: String::new() })
}

fn new_command(args: &args::NewArgs) -> Result<AppOutcome, AppError> {
    match &args.command {
        args::NewCommands::Suite(suite_args) => {
            let root = manifest_root(&suite_args.manifest);
            let request = scaffold::NewSuiteRequest {
                name: &suite_args.name,
                fixture_name: &suite_args.fixture,
                command: &suite_args.command,
            };
            let result = scaffold::create_suite(&root, &request)?;
            Ok(AppOutcome {
                exit_code: 0,
                stdout: format!(
                    "fixture: {}\napproved: {}\nmanifest:\n{}",
                    result.fixture_path.display(),
                    result.approved_path.display(),
                    result.manifest_block
                ),
                stderr: String::new(),
            })
        }
    }
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
    fixture_count: usize,
    received_path: PathBuf,
    diff_path: PathBuf,
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

    Ok(CheckResult {
        report,
        stdout,
        stderr,
        diff_text,
        fixture_count: fixtures.len(),
        received_path: stored.received_path,
        diff_path: stored.diff_path,
    })
}

fn load_suite(suite_name: &str, manifest_path: &Path) -> Result<LoadedSuite, AppError> {
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

fn json<T: Serialize>(value: &T) -> Result<String, AppError> {
    let mut text = serde_json::to_string_pretty(value)
        .map_err(|source| AppError::Json { context: "command output".to_owned(), source })?;
    text.push('\n');
    Ok(text)
}

fn render_check_error(
    output: &mut String,
    suite_name: &str,
    error: &AppError,
) -> Result<(), AppError> {
    write!(output, "suite: {suite_name}\n{error}\n")
        .map_err(|source| AppError::Manifest(format!("failed to render error: {source}")))
}

fn manifest_root(path: &Path) -> PathBuf {
    path.parent().map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

#[derive(Debug, Serialize)]
struct CheckJson {
    exit_code: u8,
    suites: Vec<CheckRecord>,
}

#[derive(Debug, Serialize)]
struct CheckRecord {
    suite: String,
    status: String,
    exit_code: u8,
    fixtures: usize,
    received: Option<String>,
    diff: Option<String>,
    error: Option<String>,
}

impl CheckRecord {
    fn from_result(suite: &str, result: &CheckResult) -> Self {
        Self {
            suite: suite.to_owned(),
            status: result.report.status.as_str().to_owned(),
            exit_code: result.report.exit_code(),
            fixtures: result.fixture_count,
            received: Some(result.received_path.to_string_lossy().into_owned()),
            diff: Some(result.diff_path.to_string_lossy().into_owned()),
            error: None,
        }
    }

    fn from_error(suite: &str, error: &AppError) -> Self {
        Self {
            suite: suite.to_owned(),
            status: "error".to_owned(),
            exit_code: 2,
            fixtures: 0,
            received: None,
            diff: None,
            error: Some(error.to_string()),
        }
    }
}

#[derive(Debug, Serialize)]
struct DiffJson<'a> {
    report: &'a diff::DiffReport,
    text: &'a str,
}

#[derive(Debug, Serialize)]
struct StatusJson {
    suites: Vec<StatusRecord>,
}

#[derive(Debug, Serialize)]
struct StatusRecord {
    suite: String,
    approved: bool,
    received: bool,
    diff: bool,
}

#[derive(Debug, Serialize)]
struct ExplainRecord {
    suite: String,
    tags: Vec<String>,
    features: Vec<String>,
    fixture_globs: Vec<String>,
    fixture_count: usize,
    fixtures: Vec<String>,
    command_argv: Vec<String>,
    ok_exit_codes: Vec<i32>,
    approved_dir: String,
    received_dir: String,
    diff_dir: String,
    approved_exists: bool,
    received_exists: bool,
    diff_exists: bool,
}

impl ExplainRecord {
    fn render_text(&self) -> Result<String, AppError> {
        let mut output = String::new();
        writeln!(&mut output, "suite: {}", self.suite).map_err(|source| {
            AppError::Manifest(format!("failed to render explain output: {source}"))
        })?;
        writeln!(&mut output, "tags: {}", comma_list(&self.tags)).map_err(|source| {
            AppError::Manifest(format!("failed to render explain output: {source}"))
        })?;
        writeln!(&mut output, "features: {}", comma_list(&self.features)).map_err(|source| {
            AppError::Manifest(format!("failed to render explain output: {source}"))
        })?;
        writeln!(&mut output, "fixtures: {}", self.fixture_count).map_err(|source| {
            AppError::Manifest(format!("failed to render explain output: {source}"))
        })?;
        writeln!(&mut output, "command: {}", self.command_argv.join(" ")).map_err(|source| {
            AppError::Manifest(format!("failed to render explain output: {source}"))
        })?;
        writeln!(&mut output, "approved: {}", self.approved_dir).map_err(|source| {
            AppError::Manifest(format!("failed to render explain output: {source}"))
        })?;
        writeln!(&mut output, "received: {}", self.received_dir).map_err(|source| {
            AppError::Manifest(format!("failed to render explain output: {source}"))
        })?;
        writeln!(&mut output, "diff: {}", self.diff_dir).map_err(|source| {
            AppError::Manifest(format!("failed to render explain output: {source}"))
        })?;
        writeln!(&mut output, "approved_exists: {}", yes_no(self.approved_exists)).map_err(
            |source| AppError::Manifest(format!("failed to render explain output: {source}")),
        )?;
        writeln!(&mut output, "received_exists: {}", yes_no(self.received_exists)).map_err(
            |source| AppError::Manifest(format!("failed to render explain output: {source}")),
        )?;
        writeln!(&mut output, "diff_exists: {}", yes_no(self.diff_exists)).map_err(|source| {
            AppError::Manifest(format!("failed to render explain output: {source}"))
        })?;
        Ok(output)
    }
}

#[derive(Debug, Serialize)]
struct DoctorJson {
    findings: Vec<doctor::Finding>,
}

fn comma_list(items: &[String]) -> String {
    if items.is_empty() { "none".to_owned() } else { items.join(",") }
}
