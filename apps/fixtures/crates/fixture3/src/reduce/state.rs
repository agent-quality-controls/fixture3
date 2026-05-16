use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::candidate::{CandidateId, FileCandidate};
use super::directory::DirectoryCandidate;

#[derive(Clone, Debug)]
pub(crate) struct ReductionState {
    original_files: Vec<FileCandidate>,
    removed_directories: BTreeSet<PathBuf>,
    removed_files: BTreeSet<CandidateId>,
}

impl ReductionState {
    pub(crate) const fn new(original_files: Vec<FileCandidate>) -> Self {
        Self {
            original_files,
            removed_directories: BTreeSet::new(),
            removed_files: BTreeSet::new(),
        }
    }

    pub(crate) fn original_files(&self) -> &[FileCandidate] {
        &self.original_files
    }

    pub(crate) const fn removed_directories(&self) -> &BTreeSet<PathBuf> {
        &self.removed_directories
    }

    pub(crate) fn remaining_files(&self) -> Vec<FileCandidate> {
        self.original_files
            .iter()
            .filter(|candidate| !self.removed_files.contains(&candidate.id()))
            .filter(|candidate| !self.is_removed_by_directory(candidate.relative_path()))
            .cloned()
            .collect()
    }

    pub(crate) fn removed_files(&self) -> Vec<FileCandidate> {
        self.original_files
            .iter()
            .filter(|candidate| {
                self.removed_files.contains(&candidate.id())
                    || self.is_removed_by_directory(candidate.relative_path())
            })
            .cloned()
            .collect()
    }

    pub(crate) fn with_removed_directories<I>(&self, directories: I) -> Self
    where
        I: IntoIterator<Item = PathBuf>,
    {
        let mut next = self.clone();
        for directory in directories {
            let _inserted = next.removed_directories.insert(directory);
        }
        next
    }

    pub(crate) fn apply_removed_directories<I>(&mut self, directories: I)
    where
        I: IntoIterator<Item = PathBuf>,
    {
        for directory in directories {
            let _inserted = self.removed_directories.insert(directory);
        }
    }

    pub(crate) fn with_file_candidates(&self, remaining: &[FileCandidate]) -> Self {
        let remaining_ids = remaining.iter().map(FileCandidate::id).collect::<BTreeSet<_>>();
        let mut next = self.clone();
        for candidate in self.remaining_files() {
            if !remaining_ids.contains(&candidate.id()) {
                let _inserted = next.removed_files.insert(candidate.id());
            }
        }
        next
    }

    pub(crate) fn apply_file_candidates(&mut self, remaining: &[FileCandidate]) {
        *self = self.with_file_candidates(remaining);
    }

    pub(crate) fn directory_is_removed(&self, directory: &DirectoryCandidate) -> bool {
        has_removed_ancestor(directory.relative_path(), &self.removed_directories)
    }

    fn is_removed_by_directory(&self, path: &Path) -> bool {
        has_removed_ancestor(path, &self.removed_directories)
    }
}

fn has_removed_ancestor(path: &Path, removed_directories: &BTreeSet<PathBuf>) -> bool {
    let mut current = Some(path);
    while let Some(candidate) = current {
        if removed_directories.contains(candidate) {
            return true;
        }
        current = candidate.parent();
    }
    false
}
