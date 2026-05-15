use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::fs;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Manifest {
    pub(crate) version: u16,
    #[serde(default)]
    pub(crate) features: BTreeMap<String, FeatureConfig>,
    pub(crate) suites: BTreeMap<String, SuiteConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct FeatureConfig {
    pub(crate) suites: Vec<String>,
    pub(crate) spec: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SuiteConfig {
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    pub(crate) fixtures: Vec<String>,
    pub(crate) command: CommandConfig,
    pub(crate) output: OutputConfig,
    pub(crate) storage: StorageConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct CommandConfig {
    pub(crate) argv: Vec<String>,
    pub(crate) ok_exit_codes: Vec<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct OutputConfig {
    pub(crate) format: OutputFormat,
    pub(crate) normalizer: Option<NormalizerConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum OutputFormat {
    Json,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct NormalizerConfig {
    pub(crate) argv: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
