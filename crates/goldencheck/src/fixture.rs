use std::path::PathBuf;

use glob::glob;
use serde::Serialize;

use crate::error::AppError;
use crate::fs;
use crate::manifest::SuiteConfig;

pub(crate) type FixturePaths = Vec<PathBuf>;
pub(crate) type FixtureRecords = Vec<FixtureRecord>;

#[derive(Clone, Debug, serde::Deserialize, Serialize)]
pub(crate) struct FixtureRecord {
    pub(crate) path: String,
    pub(crate) sha256: String,
}

pub(crate) fn discover(suite: &SuiteConfig) -> Result<FixturePaths, AppError> {
    let mut fixtures = Vec::new();

    for pattern in &suite.fixtures {
        let entries = glob(pattern).map_err(|source| {
            AppError::Manifest(format!("invalid fixture glob {pattern}: {source}"))
        })?;
        for entry in entries {
            let path = entry.map_err(|source| {
                AppError::Manifest(format!("fixture glob failed for {pattern}: {source}"))
            })?;
            fixtures.push(path);
        }
    }

    fixtures.sort();
    fixtures.dedup();

    if fixtures.is_empty() {
        return Err(AppError::Manifest("suite discovered no fixtures".to_owned()));
    }

    Ok(fixtures)
}

pub(crate) fn records(fixtures: &[PathBuf]) -> Result<FixtureRecords, AppError> {
    fixtures
        .iter()
        .map(|path| {
            let bytes = fs::read(path)?;
            Ok(FixtureRecord { path: path.to_string_lossy().into_owned(), sha256: sha256(&bytes) })
        })
        .collect()
}

fn sha256(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    use sha2::{Digest, Sha256};

    let digest = Sha256::digest(bytes);
    let mut hex = String::new();
    for byte in digest {
        if write!(&mut hex, "{byte:02x}").is_err() {
            return "sha256:".to_owned();
        }
    }
    format!("sha256:{hex}")
}
