//! `fixture3` CLI entry point.

use std::process::ExitCode;

use clap as _;
use glob as _;
use serde as _;
use serde_json as _;
use serde_norway as _;
use sha2 as _;
use thiserror as _;
use time as _;

fn main() -> ExitCode {
    fixture3_cli::run_to_stdio()
}
