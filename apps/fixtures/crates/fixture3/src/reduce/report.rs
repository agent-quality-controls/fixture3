use std::path::Path;

use fixture3_ddmin::{DdminGuarantee, DdminOutput, DdminStopReason};
use serde::Serialize;

use crate::error::AppError;

use super::candidate::FileCandidate;

#[derive(Debug, Serialize)]
pub(crate) struct ReduceReport {
    suite: String,
    fixture_root: String,
    work_dir: String,
    candidate_count: usize,
    remaining_count: usize,
    removed_count: usize,
    oracle_calls: usize,
    interesting_trials: usize,
    not_interesting_trials: usize,
    unresolved_trials: usize,
    guarantee: String,
    remaining_files: Vec<String>,
    removed_files: Vec<String>,
}

impl ReduceReport {
    pub(crate) fn from_ddmin(
        args: &crate::args::ReduceArgs,
        candidate_count: usize,
        output: &DdminOutput<FileCandidate>,
        oracle_calls: usize,
    ) -> Self {
        let stats = output.stats();
        Self {
            suite: args.suite.clone(),
            fixture_root: args.fixture_root.to_string_lossy().into_owned(),
            work_dir: args.work_dir.to_string_lossy().into_owned(),
            candidate_count,
            remaining_count: output.remaining().len(),
            removed_count: output.removed().len(),
            oracle_calls,
            interesting_trials: stats.interesting_trials(),
            not_interesting_trials: stats.not_interesting_trials(),
            unresolved_trials: stats.unresolved_trials(),
            guarantee: guarantee_text(output.guarantee()),
            remaining_files: file_list(output.remaining()),
            removed_files: file_list(output.removed()),
        }
    }

    fn removed_files(&self) -> &[String] {
        &self.removed_files
    }

    fn remaining_files(&self) -> &[String] {
        &self.remaining_files
    }
}

pub(crate) fn write(work_dir: &Path, report: &ReduceReport) -> Result<(), AppError> {
    crate::fs::create_dir_all(work_dir)?;
    let json = json(report)?;
    crate::fs::write_string(&work_dir.join("reduce-report.json"), &json)?;
    crate::fs::write_string(
        &work_dir.join("removed-files.txt"),
        &line_file(report.removed_files()),
    )?;
    crate::fs::write_string(
        &work_dir.join("remaining-files.txt"),
        &line_file(report.remaining_files()),
    )
}

fn file_list(candidates: &[FileCandidate]) -> Vec<String> {
    let mut paths = candidates.iter().map(FileCandidate::display_path).collect::<Vec<_>>();
    paths.sort();
    paths
}

fn guarantee_text(guarantee: DdminGuarantee) -> String {
    match guarantee {
        DdminGuarantee::OneMinimalWithinCandidateSet => "complete".to_owned(),
        DdminGuarantee::Incomplete(DdminStopReason::MaxOracleCallsReached) => {
            "incomplete:max-oracle-calls-reached".to_owned()
        }
        DdminGuarantee::Incomplete(DdminStopReason::BaselineNotInteresting) => {
            "incomplete:baseline-not-interesting".to_owned()
        }
    }
}

fn line_file(paths: &[String]) -> String {
    let mut text = paths.join("\n");
    if !text.is_empty() {
        text.push('\n');
    }
    text
}

fn json<T: Serialize>(value: &T) -> Result<String, AppError> {
    let mut text = serde_json::to_string_pretty(value)
        .map_err(|source| AppError::Json { context: "reducer report".to_owned(), source })?;
    text.push('\n');
    Ok(text)
}
