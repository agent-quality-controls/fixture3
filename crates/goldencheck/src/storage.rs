use serde::Serialize;
use std::path::PathBuf;

use crate::diff::DiffReport;
use crate::error::AppError;
use crate::fs;
use crate::manifest::StorageConfig;
use crate::metadata::{self, RunMetadata};

type DiffRead = (DiffReport, String);
type OptionalMetadata = Option<RunMetadata>;

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
        return Err(AppError::Manifest(format!(
            "approved output missing: {}",
            approved_path.display()
        )));
    }

    let approved = fs::read_to_string(&approved_path)?;
    if let Some(approved_metadata) =
        read_optional_metadata(&storage.approved.join("approved.meta.json"))?
    {
        metadata::assert_hashes_match(&approved_metadata, metadata)?;
    }

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
    change_path: Option<String>,
) -> Result<(), AppError> {
    let (report, _) = read_diff(storage)?;
    if report.changed && change_path.is_none() {
        return Err(AppError::Manifest(
            "approve requires --change when received output differs".to_owned(),
        ));
    }

    let received_output = fs::read(&storage.received.join("received.normalized.json"))?;
    let received_metadata: RunMetadata = read_json(&storage.received.join("received.meta.json"))?;
    let approved_metadata = metadata::approve(received_metadata, change_path);

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
suites:
  example:
    fixtures:
      - "behavior/fixtures/example/*/input.json"
    command:
      argv:
        - "cat"
        - "{fixtures}"
      ok_exit_codes:
        - 0
    output:
      format: "json"
    storage:
      approved_dir: "behavior/golden/example"
      received_dir: ".goldencheck/example"
      diff_dir: ".goldencheck/example"
"#;
    fs::write_string(path, text)
}

fn read_json<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Result<T, AppError> {
    let source = fs::read(path)?;
    serde_json::from_slice(&source)
        .map_err(|source| AppError::Json { context: path.display().to_string(), source })
}

fn read_optional_metadata(path: &std::path::Path) -> Result<OptionalMetadata, AppError> {
    if !fs::exists(path) {
        return Ok(None);
    }
    Ok(Some(read_json(path)?))
}
