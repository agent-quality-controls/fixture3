use std::path::PathBuf;
use std::process::Stdio;

use crate::error::AppError;
use crate::manifest::CommandConfig;

const FIXTURES_TOKEN: &str = "{fixtures}";

#[derive(Debug)]
pub(crate) struct CommandOutput {
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
    pub(crate) exit_code: i32,
}

pub(crate) fn run_fixture_command(
    config: &CommandConfig,
    fixtures: &[PathBuf],
) -> Result<CommandOutput, AppError> {
    let argv = expand_fixture_args(&config.argv, fixtures);
    let output = run_argv(&argv)?;

    if !config.ok_exit_codes.contains(&output.exit_code) {
        return Err(AppError::Command(format!(
            "exit code {} was not in {:?}",
            output.exit_code, config.ok_exit_codes
        )));
    }

    Ok(output)
}

pub(crate) fn run_program(program: &str, arguments: &[&str]) -> Result<CommandOutput, AppError> {
    let command_argv = std::iter::once(program.to_owned())
        .chain(arguments.iter().map(|arg| (*arg).to_owned()))
        .collect::<Vec<_>>();
    run_argv(&command_argv)
}

fn expand_fixture_args(argv: &[String], fixtures: &[PathBuf]) -> Vec<String> {
    let fixture_args =
        fixtures.iter().map(|path| path.to_string_lossy().into_owned()).collect::<Vec<_>>();

    let mut expanded = Vec::new();
    for arg in argv {
        if arg == FIXTURES_TOKEN {
            expanded.extend(fixture_args.clone());
        } else {
            expanded.push(arg.replace(FIXTURES_TOKEN, &fixture_args.join(" ")));
        }
    }
    expanded
}

#[allow(clippy::disallowed_methods, reason = "This module is the command adapter.")]
fn run_argv(argv: &[String]) -> Result<CommandOutput, AppError> {
    let Some((program, arguments)) = argv.split_first() else {
        return Err(AppError::Command("empty argv".to_owned()));
    };

    let output = std::process::Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|source| AppError::Command(format!("failed to wait for {program}: {source}")))?;

    Ok(CommandOutput {
        stdout: output.stdout,
        stderr: output.stderr,
        exit_code: output.status.code().unwrap_or(2),
    })
}
