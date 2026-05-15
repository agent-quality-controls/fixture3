use std::path::Path;

use crate::error::AppError;

pub(crate) type Bytes = Vec<u8>;

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
pub(crate) fn exists(path: &Path) -> bool {
    path.exists()
}
