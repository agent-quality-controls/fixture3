use std::path::PathBuf;
use std::string::FromUtf8Error;

use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum AppError {
    #[error("command failed: {0}")]
    Command(String),

    #[error("filesystem error at {path}: {source}")]
    Filesystem { path: PathBuf, source: std::io::Error },

    #[error("git error: {0}")]
    Git(String),

    #[error("json error in {context}: {source}")]
    Json { context: String, source: serde_json::Error },

    #[error("manifest error: {0}")]
    Manifest(String),

    #[error("yaml error at {path}: {source}")]
    Yaml { path: PathBuf, source: serde_norway::Error },

    #[error("utf8 error in {context}: {source}")]
    Utf8 { context: String, source: FromUtf8Error },
}

impl AppError {
    pub(crate) fn fs(path: &std::path::Path, source: std::io::Error) -> Self {
        Self::Filesystem { path: path.to_path_buf(), source }
    }
}
