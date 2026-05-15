use crate::command;
use crate::error::AppError;

#[derive(Debug)]
pub(crate) struct GitState {
    pub(crate) commit: String,
    pub(crate) working_tree: String,
}

pub(crate) fn state() -> Result<GitState, AppError> {
    let commit_output = command::run_program("git", &["rev-parse", "--short", "HEAD"])?;
    if commit_output.exit_code != 0 {
        return Err(AppError::Git("failed to read HEAD commit".to_owned()));
    }

    let status_output = command::run_program("git", &["status", "--porcelain"])?;
    if status_output.exit_code != 0 {
        return Err(AppError::Git("failed to read working tree status".to_owned()));
    }

    let commit = String::from_utf8(commit_output.stdout)
        .map_err(|source| AppError::Utf8 { context: "git commit".to_owned(), source })?
        .trim()
        .to_owned();
    let status = String::from_utf8(status_output.stdout)
        .map_err(|source| AppError::Utf8 { context: "git status".to_owned(), source })?;
    let working_tree = if status.trim().is_empty() { "clean" } else { "dirty" }.to_owned();

    Ok(GitState { commit, working_tree })
}
