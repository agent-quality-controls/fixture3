use std::path::Path;

use fixture3_ddmin::{DdminGuarantee, DdminOutput, DdminStopReason, OracleOutcome};
use serde::Serialize;

use crate::error::AppError;

use super::candidate::FileCandidate;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ReduceProgress {
    oracle_calls: usize,
    interesting_trials: usize,
    not_interesting_trials: usize,
    unresolved_trials: usize,
}

impl ReduceProgress {
    pub(crate) const fn recorded(mut self, outcome: &OracleOutcome) -> Self {
        self.oracle_calls = self.oracle_calls.saturating_add(1);
        match outcome {
            OracleOutcome::Interesting => {
                self.interesting_trials = self.interesting_trials.saturating_add(1);
            }
            OracleOutcome::NotInteresting => {
                self.not_interesting_trials = self.not_interesting_trials.saturating_add(1);
            }
            OracleOutcome::Unresolved(_) => {
                self.unresolved_trials = self.unresolved_trials.saturating_add(1);
            }
        }
        self
    }

    pub(crate) const fn oracle_calls(self) -> usize {
        self.oracle_calls
    }

    const fn interesting_trials(self) -> usize {
        self.interesting_trials
    }

    const fn not_interesting_trials(self) -> usize {
        self.not_interesting_trials
    }

    const fn unresolved_trials(self) -> usize {
        self.unresolved_trials
    }
}

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

    pub(crate) fn best_so_far(
        suite: &str,
        fixture_root: &Path,
        work_dir: &Path,
        original: &[FileCandidate],
        remaining: &[FileCandidate],
        progress: ReduceProgress,
    ) -> Self {
        let removed = removed_from_original(original, remaining);
        Self {
            suite: suite.to_owned(),
            fixture_root: fixture_root.to_string_lossy().into_owned(),
            work_dir: work_dir.to_string_lossy().into_owned(),
            candidate_count: original.len(),
            remaining_count: remaining.len(),
            removed_count: removed.len(),
            oracle_calls: progress.oracle_calls(),
            interesting_trials: progress.interesting_trials(),
            not_interesting_trials: progress.not_interesting_trials(),
            unresolved_trials: progress.unresolved_trials(),
            guarantee: "best-so-far".to_owned(),
            remaining_files: file_list(remaining),
            removed_files: file_list(&removed),
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

pub(crate) fn write_best(work_dir: &Path, report: &ReduceReport) -> Result<(), AppError> {
    write(&work_dir.join("best"), report)
}

fn file_list(candidates: &[FileCandidate]) -> Vec<String> {
    let mut paths = candidates.iter().map(FileCandidate::display_path).collect::<Vec<_>>();
    paths.sort();
    paths
}

fn removed_from_original(
    original: &[FileCandidate],
    remaining: &[FileCandidate],
) -> Vec<FileCandidate> {
    let remaining_ids =
        remaining.iter().map(FileCandidate::id).collect::<std::collections::BTreeSet<_>>();
    original.iter().filter(|candidate| !remaining_ids.contains(&candidate.id())).cloned().collect()
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
