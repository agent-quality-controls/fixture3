use std::path::Path;

use fixture3_ddmin::{DdminGuarantee, DdminStopReason, OracleOutcome};
use serde::Serialize;

use crate::error::AppError;

use super::candidate::FileCandidate;
use super::state::ReductionState;

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

    pub(crate) const fn interesting_trials(self) -> usize {
        self.interesting_trials
    }

    pub(crate) const fn not_interesting_trials(self) -> usize {
        self.not_interesting_trials
    }

    pub(crate) const fn unresolved_trials(self) -> usize {
        self.unresolved_trials
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct PhaseReport {
    reducer: String,
    candidate_count: usize,
    remaining_count: usize,
    removed_count: usize,
    guarantee: String,
}

impl PhaseReport {
    pub(crate) fn new(
        reducer: &str,
        candidate_count: usize,
        remaining_count: usize,
        guarantee: String,
    ) -> Self {
        Self {
            reducer: reducer.to_owned(),
            candidate_count,
            remaining_count,
            removed_count: candidate_count.saturating_sub(remaining_count),
            guarantee,
        }
    }

    pub(crate) fn guarantee(&self) -> &str {
        &self.guarantee
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ReportContext {
    reducers: Vec<String>,
    phases: Vec<PhaseReport>,
    directory_candidate_count: usize,
}

impl ReportContext {
    pub(crate) const fn new(reducers: Vec<String>, directory_candidate_count: usize) -> Self {
        Self { reducers, phases: Vec::new(), directory_candidate_count }
    }

    pub(crate) fn with_phases(&self, phases: Vec<PhaseReport>) -> Self {
        Self {
            reducers: self.reducers.clone(),
            phases,
            directory_candidate_count: self.directory_candidate_count,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct ReduceReport {
    suite: String,
    fixture_root: String,
    work_dir: String,
    reducers: Vec<String>,
    phases: Vec<PhaseReport>,
    candidate_count: usize,
    directory_candidate_count: usize,
    remaining_count: usize,
    removed_count: usize,
    oracle_calls: usize,
    interesting_trials: usize,
    not_interesting_trials: usize,
    unresolved_trials: usize,
    guarantee: String,
    remaining_files: Vec<String>,
    removed_files: Vec<String>,
    removed_directories: Vec<String>,
}

impl ReduceReport {
    pub(crate) fn from_state(
        args: &crate::args::ReduceArgs,
        context: &ReportContext,
        state: &ReductionState,
        progress: ReduceProgress,
        guarantee: String,
    ) -> Self {
        let remaining = state.remaining_files();
        let removed = state.removed_files();
        Self {
            suite: args.suite.clone(),
            fixture_root: args.fixture_root.to_string_lossy().into_owned(),
            work_dir: args.work_dir.to_string_lossy().into_owned(),
            reducers: context.reducers.clone(),
            phases: context.phases.clone(),
            candidate_count: state.original_files().len(),
            directory_candidate_count: context.directory_candidate_count,
            remaining_count: remaining.len(),
            removed_count: removed.len(),
            oracle_calls: progress.oracle_calls(),
            interesting_trials: progress.interesting_trials(),
            not_interesting_trials: progress.not_interesting_trials(),
            unresolved_trials: progress.unresolved_trials(),
            guarantee,
            remaining_files: file_list(&remaining),
            removed_files: file_list(&removed),
            removed_directories: directory_list(state),
        }
    }

    pub(crate) fn best_so_far(
        suite: &str,
        fixture_root: &Path,
        work_dir: &Path,
        context: &ReportContext,
        state: &ReductionState,
        progress: ReduceProgress,
    ) -> Self {
        let remaining = state.remaining_files();
        let removed = state.removed_files();
        Self {
            suite: suite.to_owned(),
            fixture_root: fixture_root.to_string_lossy().into_owned(),
            work_dir: work_dir.to_string_lossy().into_owned(),
            reducers: context.reducers.clone(),
            phases: context.phases.clone(),
            candidate_count: state.original_files().len(),
            directory_candidate_count: context.directory_candidate_count,
            remaining_count: remaining.len(),
            removed_count: removed.len(),
            oracle_calls: progress.oracle_calls(),
            interesting_trials: progress.interesting_trials(),
            not_interesting_trials: progress.not_interesting_trials(),
            unresolved_trials: progress.unresolved_trials(),
            guarantee: "best-so-far".to_owned(),
            remaining_files: file_list(&remaining),
            removed_files: file_list(&removed),
            removed_directories: directory_list(state),
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

pub(crate) fn guarantee_text(guarantee: DdminGuarantee) -> String {
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

fn directory_list(state: &ReductionState) -> Vec<String> {
    state.removed_directories().iter().map(|path| path.to_string_lossy().into_owned()).collect()
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
