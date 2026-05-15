use crate::error::AppError;

use super::candidate::FileCandidate;
use super::trial_tree::TrialTree;

pub(crate) fn write_trial_manifest(
    original_manifest: &std::path::Path,
    suite_name: &str,
    trial: &TrialTree,
    remaining: &[FileCandidate],
) -> Result<(), AppError> {
    let mut manifest = crate::manifest::load(original_manifest)?;
    let suite = manifest
        .suites
        .get_mut(suite_name)
        .ok_or_else(|| AppError::Manifest(format!("suite not found in manifest: {suite_name}")))?;

    suite.fixtures = remaining
        .iter()
        .map(|candidate| {
            trial.fixture_root().join(candidate.relative_path()).to_string_lossy().into_owned()
        })
        .collect();
    suite.storage.received = trial.received_dir().to_path_buf();
    suite.storage.diff = trial.diff_dir().to_path_buf();

    let text = serde_norway::to_string(&manifest)
        .map_err(|source| AppError::Yaml { path: trial.manifest_path().to_path_buf(), source })?;
    crate::fs::write_string(trial.manifest_path(), &text)
}
