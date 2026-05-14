use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::fs;

#[derive(Debug)]
pub(crate) struct NewSuiteRequest<'a> {
    pub(crate) name: &'a str,
    pub(crate) fixture_name: &'a str,
    pub(crate) command: &'a str,
}

#[derive(Debug)]
pub(crate) struct NewSuiteResult {
    pub(crate) fixture_path: PathBuf,
    pub(crate) approved_path: PathBuf,
    pub(crate) manifest_block: String,
}

pub(crate) fn create_suite(
    root: &Path,
    request: &NewSuiteRequest<'_>,
) -> Result<NewSuiteResult, AppError> {
    let fixture_path = root
        .join("behavior")
        .join("fixtures")
        .join(request.name)
        .join("example")
        .join(request.fixture_name);
    let approved_path =
        root.join("behavior").join("approved").join(request.name).join("approved.normalized.json");

    if fs::exists(&fixture_path) {
        return Err(AppError::Manifest(format!(
            "fixture already exists: {}",
            fixture_path.display()
        )));
    }
    if fs::exists(&approved_path) {
        return Err(AppError::Manifest(format!(
            "approved output already exists: {}",
            approved_path.display()
        )));
    }

    fs::write_string(&fixture_path, "{}\n")?;
    fs::write_string(&approved_path, "{}\n")?;

    let manifest_block = format!(
        "{name}:\n  tags: []\n  fixtures:\n    - \"behavior/fixtures/{name}/*/{fixture}\"\n  command:\n    argv:\n      - \"{command}\"\n      - \"{{fixtures}}\"\n    ok_exit_codes:\n      - 0\n  output:\n    format: \"json\"\n  storage:\n    approved_dir: \"behavior/approved/{name}\"\n    received_dir: \".fixture3/{name}\"\n    diff_dir: \".fixture3/{name}\"\n",
        name = request.name,
        fixture = request.fixture_name,
        command = request.command
    );

    Ok(NewSuiteResult { fixture_path, approved_path, manifest_block })
}
