use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::error::AppError;
use crate::fixture::{self, FixtureRecord};
use crate::fs;
use crate::git;
use crate::manifest::SuiteConfig;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct RunMetadata {
    pub(crate) suite: String,
    pub(crate) kind: String,
    pub(crate) run_id: String,
    pub(crate) run_commit: String,
    pub(crate) working_tree: String,
    pub(crate) recorded_at: String,
    pub(crate) fixture_hash: String,
    pub(crate) manifest_hash: String,
    pub(crate) normalizer_hash: String,
    pub(crate) tool_version: String,
    pub(crate) output_schema_version: String,
    pub(crate) fixtures: Vec<FixtureRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) change_path: Option<String>,
}

pub(crate) fn build(
    suite_name: &str,
    suite: &SuiteConfig,
    manifest_path: &Path,
    fixture_paths: &[PathBuf],
) -> Result<RunMetadata, AppError> {
    let git_state = git::state()?;
    let recorded_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|source| AppError::Manifest(format!("failed to format timestamp: {source}")))?;
    let fixtures = fixture::records(fixture_paths)?;
    let fixture_hash = hash_fixture_set(&fixtures)?;
    let manifest_hash = hash_bytes(&fs::read(manifest_path)?);
    let normalizer_hash = hash_bytes(format!("{:?}", suite.output).as_bytes());
    let manifest_suffix = manifest_hash.chars().skip(7).take(8).collect::<String>();
    let run_id = format!("{}-{}", recorded_at.replace(':', "-"), manifest_suffix);

    Ok(RunMetadata {
        suite: suite_name.to_owned(),
        kind: "received".to_owned(),
        run_id,
        run_commit: git_state.commit,
        working_tree: git_state.working_tree,
        recorded_at,
        fixture_hash,
        manifest_hash,
        normalizer_hash,
        tool_version: env!("CARGO_PKG_VERSION").to_owned(),
        output_schema_version: "1".to_owned(),
        fixtures,
        change_path: None,
    })
}

pub(crate) fn approve(mut metadata: RunMetadata, change_path: Option<String>) -> RunMetadata {
    "approved".clone_into(&mut metadata.kind);
    metadata.change_path = change_path;
    metadata
}

pub(crate) fn assert_hashes_match(
    approved: &RunMetadata,
    received: &RunMetadata,
) -> Result<(), AppError> {
    let checks = [
        ("fixture", &approved.fixture_hash, &received.fixture_hash),
        ("manifest", &approved.manifest_hash, &received.manifest_hash),
        ("normalizer", &approved.normalizer_hash, &received.normalizer_hash),
    ];

    for (name, approved_hash, received_hash) in checks {
        if approved_hash != received_hash {
            return Err(AppError::Manifest(format!(
                "{name} hash changed: approved {approved_hash}, received {received_hash}"
            )));
        }
    }

    Ok(())
}

fn hash_fixture_set(fixtures: &[FixtureRecord]) -> Result<String, AppError> {
    let bytes = serde_json::to_vec(fixtures)
        .map_err(|source| AppError::Json { context: "fixture metadata".to_owned(), source })?;
    Ok(hash_bytes(&bytes))
}

fn hash_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::new();
    for byte in digest {
        if write!(&mut hex, "{byte:02x}").is_err() {
            return "sha256:".to_owned();
        }
    }
    format!("sha256:{hex}")
}
