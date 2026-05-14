//! `fixture3` command library.

pub(crate) mod app;
pub(crate) mod args;
pub(crate) mod command;
pub(crate) mod diff;
pub(crate) mod doctor;
pub(crate) mod error;
pub(crate) mod fixture;
pub(crate) mod fs;
pub(crate) mod git;
pub(crate) mod manifest;
pub(crate) mod metadata;
pub(crate) mod normalize;
pub(crate) mod scaffold;
pub(crate) mod selection;
pub(crate) mod storage;

#[cfg(feature = "cli")]
pub use app::run_to_stdio;
