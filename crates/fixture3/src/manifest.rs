use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::AppError;
use crate::fs;

#[derive(Debug, Deserialize)]
pub(crate) struct Manifest {
    pub(crate) version: u16,
    pub(crate) suites: BTreeMap<String, SuiteConfig>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SuiteConfig {
    pub(crate) fixtures: Vec<String>,
    pub(crate) command: CommandConfig,
    pub(crate) output: OutputConfig,
    pub(crate) storage: StorageConfig,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CommandConfig {
    pub(crate) argv: Vec<String>,
    pub(crate) ok_exit_codes: Vec<i32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OutputConfig {
    pub(crate) format: OutputFormat,
    pub(crate) normalizer: Option<NormalizerConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum OutputFormat {
    Json,
}

#[derive(Debug, Deserialize)]
pub(crate) struct NormalizerConfig {
    pub(crate) argv: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct StorageConfig {
    #[serde(rename = "approved_dir")]
    pub(crate) approved: PathBuf,
    #[serde(rename = "received_dir")]
    pub(crate) received: PathBuf,
    #[serde(rename = "diff_dir")]
    pub(crate) diff: PathBuf,
}

pub(crate) fn load(path: &Path) -> Result<Manifest, AppError> {
    let source = fs::read_to_string(path)?;
    let manifest: Manifest = serde_norway::from_str(&source)
        .map_err(|source| AppError::Yaml { path: path.to_path_buf(), source })?;

    if manifest.version != 1 {
        return Err(AppError::Manifest(format!(
            "unsupported manifest version {}",
            manifest.version
        )));
    }

    Ok(manifest)
}
