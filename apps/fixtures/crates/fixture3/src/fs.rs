use std::path::Path;

use crate::error::AppError;

pub(crate) type Bytes = Vec<u8>;
pub(crate) type DirEntries = Vec<std::fs::DirEntry>;

#[allow(clippy::disallowed_methods, reason = "This module is the filesystem adapter.")]
pub(crate) fn read(path: &Path) -> Result<Bytes, AppError> {
    std::fs::read(path).map_err(|source| AppError::fs(path, source))
}

#[allow(clippy::disallowed_methods, reason = "This module is the filesystem adapter.")]
pub(crate) fn read_to_string(path: &Path) -> Result<String, AppError> {
    std::fs::read_to_string(path).map_err(|source| AppError::fs(path, source))
}

#[allow(clippy::disallowed_methods, reason = "This module is the filesystem adapter.")]
pub(crate) fn write(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| AppError::fs(parent, source))?;
    }
    std::fs::write(path, bytes).map_err(|source| AppError::fs(path, source))
}

pub(crate) fn write_string(path: &Path, text: &str) -> Result<(), AppError> {
    write(path, text.as_bytes())
}

#[allow(clippy::disallowed_methods, reason = "This module is the filesystem adapter.")]
pub(crate) fn create_dir_all(path: &Path) -> Result<(), AppError> {
    std::fs::create_dir_all(path).map_err(|source| AppError::fs(path, source))
}

#[allow(clippy::disallowed_methods, reason = "This module is the filesystem adapter.")]
pub(crate) fn remove_dir_all(path: &Path) -> Result<(), AppError> {
    if !path.exists() {
        return Ok(());
    }
    std::fs::remove_dir_all(path).map_err(|source| AppError::fs(path, source))
}

#[allow(clippy::disallowed_methods, reason = "This module is the filesystem adapter.")]
pub(crate) fn copy(from: &Path, to: &Path) -> Result<(), AppError> {
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent).map_err(|source| AppError::fs(parent, source))?;
    }
    let _bytes_copied = std::fs::copy(from, to).map_err(|source| AppError::fs(to, source))?;
    Ok(())
}

#[allow(clippy::disallowed_methods, reason = "This module is the filesystem adapter.")]
pub(crate) fn read_dir(path: &Path) -> Result<DirEntries, AppError> {
    let entries = std::fs::read_dir(path).map_err(|source| AppError::fs(path, source))?;
    entries
        .map(|entry| entry.map_err(|source| AppError::fs(path, source)))
        .collect::<Result<Vec<_>, _>>()
}

#[allow(clippy::disallowed_methods, reason = "This module is the filesystem adapter.")]
pub(crate) fn symlink_metadata(path: &Path) -> Result<std::fs::Metadata, AppError> {
    std::fs::symlink_metadata(path).map_err(|source| AppError::fs(path, source))
}

#[allow(clippy::disallowed_methods, reason = "This module is the filesystem adapter.")]
pub(crate) fn exists(path: &Path) -> bool {
    path.exists()
}
