use std::path::{Component, Path, PathBuf};

use crate::error::AppError;

pub(crate) type FileCandidates = Vec<FileCandidate>;

const EXCLUDED_COMPONENTS: &[&str] =
    &[".git", "target", ".cargo-target", "node_modules", "dist", ".fixture3", ".goldencheck"];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct CandidateId(u32);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FileCandidate {
    id: CandidateId,
    relative_path: PathBuf,
}

impl CandidateId {
    const fn new(value: u32) -> Self {
        Self(value)
    }

    pub(crate) const fn get(self) -> u32 {
        self.0
    }
}

impl FileCandidate {
    pub(crate) const fn id(&self) -> CandidateId {
        self.id
    }

    pub(crate) fn relative_path(&self) -> &Path {
        &self.relative_path
    }

    pub(crate) fn display_path(&self) -> String {
        self.relative_path.to_string_lossy().into_owned()
    }
}

pub(crate) fn collect(root: &Path) -> Result<FileCandidates, AppError> {
    let metadata = crate::fs::symlink_metadata(root)?;
    if metadata.file_type().is_symlink() {
        return Err(symlink_error(root));
    }
    if !metadata.is_dir() {
        return Err(AppError::Manifest(format!(
            "fixture-root is not a directory: {}",
            root.display()
        )));
    }

    let mut paths = Vec::new();
    collect_paths(root, root, &mut paths)?;
    paths.sort();

    paths
        .into_iter()
        .enumerate()
        .map(|(index, relative_path)| {
            let id = u32::try_from(index)
                .map_err(|_| AppError::Manifest("too many fixture candidates".to_owned()))?;
            Ok(FileCandidate { id: CandidateId::new(id), relative_path })
        })
        .collect()
}

fn collect_paths(root: &Path, directory: &Path, paths: &mut Vec<PathBuf>) -> Result<(), AppError> {
    for entry in crate::fs::read_dir(directory)? {
        let path = entry.path();
        let metadata = crate::fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            return Err(symlink_error(&path));
        }

        let relative = path.strip_prefix(root).map_err(|source| {
            AppError::Manifest(format!("fixture path escaped root: {}: {source}", path.display()))
        })?;
        if has_excluded_component(relative) {
            continue;
        }
        if has_unsafe_component(relative) {
            return Err(AppError::Manifest(format!(
                "fixture path escaped root: {}",
                relative.display()
            )));
        }

        if metadata.is_dir() {
            collect_paths(root, &path, paths)?;
        } else if metadata.is_file() {
            paths.push(relative.to_path_buf());
        }
    }
    Ok(())
}

fn has_excluded_component(path: &Path) -> bool {
    path.components().any(|component| {
        let Component::Normal(name) = component else {
            return false;
        };
        EXCLUDED_COMPONENTS.iter().any(|excluded| name == *excluded)
    })
}

fn has_unsafe_component(path: &Path) -> bool {
    path.components()
        .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
}

fn symlink_error(path: &Path) -> AppError {
    AppError::Manifest(format!("symlink fixture content is not supported: {}", path.display()))
}
