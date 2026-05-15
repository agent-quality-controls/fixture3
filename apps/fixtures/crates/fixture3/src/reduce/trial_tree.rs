use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::error::AppError;

use super::candidate::FileCandidate;

#[derive(Debug)]
pub(crate) struct TrialTree {
    fixture_root: PathBuf,
    manifest_path: PathBuf,
    received_dir: PathBuf,
    diff_dir: PathBuf,
}

impl TrialTree {
    pub(crate) fn fixture_root(&self) -> &Path {
        &self.fixture_root
    }

    pub(crate) fn manifest_path(&self) -> &Path {
        &self.manifest_path
    }

    pub(crate) fn received_dir(&self) -> &Path {
        &self.received_dir
    }

    pub(crate) fn diff_dir(&self) -> &Path {
        &self.diff_dir
    }
}

pub(crate) fn create(
    fixture_root: &Path,
    work_dir: &Path,
    remaining: &[FileCandidate],
) -> Result<TrialTree, AppError> {
    let call_dir = work_dir.join("trial-current");
    crate::fs::remove_dir_all(&call_dir)?;

    let trial = TrialTree {
        fixture_root: call_dir.join("fixture-root"),
        manifest_path: call_dir.join("manifest.fixture3.yaml"),
        received_dir: call_dir.join("received"),
        diff_dir: call_dir.join("diff"),
    };
    crate::fs::create_dir_all(trial.fixture_root())?;

    let mut seen = BTreeSet::new();
    for candidate in remaining {
        if !seen.insert(candidate.id()) {
            return Err(AppError::Manifest(format!(
                "duplicate reducer candidate id: {}",
                candidate.id().get()
            )));
        }
        copy_candidate(fixture_root, trial.fixture_root(), candidate)?;
    }

    Ok(trial)
}

fn copy_candidate(
    source_root: &Path,
    target_root: &Path,
    candidate: &FileCandidate,
) -> Result<(), AppError> {
    let source = source_root.join(candidate.relative_path());
    let target = target_root.join(candidate.relative_path());
    crate::fs::copy(&source, &target)
}
