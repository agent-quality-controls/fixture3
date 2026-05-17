use serde::Serialize;
use std::path::PathBuf;

use crate::diff::DiffReport;
use crate::error::AppError;
use crate::fs;
use crate::manifest::StorageConfig;
use crate::metadata::{self, RunMetadata};

type DiffRead = (DiffReport, String);

#[derive(Debug)]
pub(crate) struct StoredRun {
    pub(crate) approved: String,
    pub(crate) received_path: PathBuf,
    pub(crate) diff_path: PathBuf,
}

#[derive(Debug)]
pub(crate) struct SuiteStatus {
    pub(crate) approved_exists: bool,
    pub(crate) received_exists: bool,
    pub(crate) diff_exists: bool,
}

pub(crate) fn write_received(
    storage: &StorageConfig,
    raw: &[u8],
    normalized: &str,
    metadata: &RunMetadata,
) -> Result<StoredRun, AppError> {
    let approved_path = storage.approved.join("approved.normalized.json");
    if !fs::exists(&approved_path) {
        fs::write_string(&approved_path, "{}\n")?;
    }

    let approved = fs::read_to_string(&approved_path)?;
    let raw_path = storage.received.join("received.raw.json");
    let normalized_path = storage.received.join("received.normalized.json");
    let metadata_path = storage.received.join("received.meta.json");

    fs::write(&raw_path, raw)?;
    fs::write_string(&normalized_path, normalized)?;
    write_json(&metadata_path, metadata)?;

    Ok(StoredRun {
        approved,
        received_path: normalized_path,
        diff_path: storage.diff.join("diff.txt"),
    })
}

pub(crate) fn write_diff(
    storage: &StorageConfig,
    report: &DiffReport,
    text: &str,
) -> Result<(), AppError> {
    write_json(&storage.diff.join("diff.json"), report)?;
    fs::write_string(&storage.diff.join("diff.txt"), text)
}

fn write_json<T: Serialize>(path: &std::path::Path, value: &T) -> Result<(), AppError> {
    let mut text = serde_json::to_string_pretty(value)
        .map_err(|source| AppError::Json { context: path.display().to_string(), source })?;
    text.push('\n');
    fs::write_string(path, &text)
}

pub(crate) fn read_diff(storage: &StorageConfig) -> Result<DiffRead, AppError> {
    let report = read_json(&storage.diff.join("diff.json"))?;
    let text = fs::read_to_string(&storage.diff.join("diff.txt"))?;
    Ok((report, text))
}

pub(crate) fn approve_received(
    storage: &StorageConfig,
    comment: Option<String>,
) -> Result<(), AppError> {
    let received_output = fs::read(&storage.received.join("received.normalized.json"))?;
    let received_metadata: RunMetadata = read_json(&storage.received.join("received.meta.json"))?;
    let approved_metadata = metadata::approve(received_metadata, comment);

    fs::write(&storage.approved.join("approved.normalized.json"), &received_output)?;
    write_json(&storage.approved.join("approved.meta.json"), &approved_metadata)
}

pub(crate) fn status(storage: &StorageConfig) -> SuiteStatus {
    SuiteStatus {
        approved_exists: fs::exists(&storage.approved.join("approved.normalized.json")),
        received_exists: fs::exists(&storage.received.join("received.normalized.json")),
        diff_exists: fs::exists(&storage.diff.join("diff.json")),
    }
}

pub(crate) fn init_manifest(path: &std::path::Path) -> Result<(), AppError> {
    if fs::exists(path) {
        return Err(AppError::Manifest(format!("manifest already exists: {}", path.display())));
    }

    let text = r#"version: 1
features:
  example:
    spec: "docs/features/example.md"
    suites:
      - "example"
suites:
  example:
    tags:
      - "example"
    fixtures:
      - "behavior/fixtures/example/*/input.json"
    command:
      argv:
        - "cat"
        - "{fixtures}"
      ok_exit_codes:
        - 0
    storage:
      approved_dir: "behavior/approved/example"
      received_dir: ".fixture3/example"
      diff_dir: ".fixture3/example"
"#;
    fs::write_string(path, text)
}

#[allow(
    clippy::disallowed_methods,
    reason = "read_json is the storage JSON boundary for fixture3 metadata and diff files"
)]
fn read_json<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Result<T, AppError> {
    let source = fs::read(path)?;
    serde_json::from_slice(&source)
        .map_err(|source| AppError::Json { context: path.display().to_string(), source })
}
