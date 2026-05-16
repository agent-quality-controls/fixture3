use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::AppError;

use super::candidate::{CandidateId, FileCandidate};

pub(crate) type DirectoryCandidates = Vec<DirectoryCandidate>;
type DirectoryGroups = BTreeMap<usize, DirectoryCandidates>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DirectoryCandidate {
    id: CandidateId,
    relative_path: PathBuf,
    file_count: usize,
}

impl DirectoryCandidate {
    pub(crate) const fn id(&self) -> CandidateId {
        self.id
    }

    pub(crate) fn relative_path(&self) -> &Path {
        &self.relative_path
    }

    pub(crate) const fn file_count(&self) -> usize {
        self.file_count
    }

    pub(crate) fn depth(&self) -> usize {
        self.relative_path.components().count()
    }
}

pub(crate) fn collect(files: &[FileCandidate]) -> Result<DirectoryCandidates, AppError> {
    let mut counts = BTreeMap::<PathBuf, usize>::new();
    for file in files {
        for ancestor in ancestors(file.relative_path()) {
            let count = counts.entry(ancestor).or_default();
            *count = count.saturating_add(1);
        }
    }

    counts
        .into_iter()
        .enumerate()
        .map(|(index, (relative_path, file_count))| {
            let id = u32::try_from(index)
                .map_err(|_| AppError::Manifest("too many directory candidates".to_owned()))?;
            Ok(DirectoryCandidate { id: CandidateId::new(id), relative_path, file_count })
        })
        .collect()
}

pub(crate) fn by_depth(candidates: &[DirectoryCandidate]) -> DirectoryGroups {
    let mut groups = DirectoryGroups::new();
    for candidate in candidates {
        groups.entry(candidate.depth()).or_default().push(candidate.clone());
    }
    groups
}

fn ancestors(path: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut current = path.parent();
    while let Some(parent) = current {
        if parent.as_os_str().is_empty() {
            break;
        }
        result.push(parent.to_path_buf());
        current = parent.parent();
    }
    result.reverse();
    result
}
