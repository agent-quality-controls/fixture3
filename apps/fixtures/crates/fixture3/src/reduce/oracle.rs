use std::path::PathBuf;

use fixture3_ddmin::{OracleOutcome, UnresolvedReason};

use super::candidate::FileCandidate;
use super::report::{ReduceProgress, ReportContext};
use super::state::ReductionState;

#[derive(Debug)]
pub(crate) struct ReduceOracle {
    suite: String,
    manifest_path: PathBuf,
    fixture_root: PathBuf,
    work_dir: PathBuf,
    best_state: Option<ReductionState>,
    progress: ReduceProgress,
    calls: usize,
}

impl ReduceOracle {
    pub(crate) fn new(
        suite: String,
        manifest_path: PathBuf,
        fixture_root: PathBuf,
        work_dir: PathBuf,
        _original: Vec<FileCandidate>,
    ) -> Self {
        Self {
            suite,
            manifest_path,
            fixture_root,
            work_dir,
            best_state: None,
            progress: ReduceProgress::default(),
            calls: 0,
        }
    }

    pub(crate) const fn calls(&self) -> usize {
        self.calls
    }

    pub(crate) const fn progress(&self) -> ReduceProgress {
        self.progress
    }

    pub(crate) fn evaluate_state(
        &mut self,
        state: &ReductionState,
        context: &ReportContext,
    ) -> OracleOutcome {
        self.calls = self.calls.saturating_add(1);
        let remaining = state.remaining_files();
        let Ok(trial) = super::trial_tree::create(&self.fixture_root, &self.work_dir, &remaining)
        else {
            return self.recorded(OracleOutcome::Unresolved(UnresolvedReason::OracleFailed));
        };
        if super::rewrite::write_trial_manifest(
            &self.manifest_path,
            &self.suite,
            &trial,
            &remaining,
        )
        .is_err()
        {
            return self.recorded(OracleOutcome::Unresolved(UnresolvedReason::OracleFailed));
        }

        let outcome = match crate::app::run_check(&self.suite, trial.manifest_path()) {
            Ok(result) if result.report.exit_code() == 0 => OracleOutcome::Interesting,
            Ok(_) => OracleOutcome::NotInteresting,
            Err(_) => OracleOutcome::Unresolved(UnresolvedReason::OracleFailed),
        };

        if matches!(outcome, OracleOutcome::Interesting)
            && self.write_best(state, context, self.progress.recorded(&outcome)).is_err()
        {
            return self.recorded(OracleOutcome::Unresolved(UnresolvedReason::OracleFailed));
        }

        self.recorded(outcome)
    }
}

impl ReduceOracle {
    const fn recorded(&mut self, outcome: OracleOutcome) -> OracleOutcome {
        self.progress = self.progress.recorded(&outcome);
        outcome
    }

    fn write_best(
        &mut self,
        state: &ReductionState,
        context: &ReportContext,
        progress: ReduceProgress,
    ) -> Result<(), crate::error::AppError> {
        self.best_state = Some(state.clone());
        let best_state = self.best_state.as_ref().ok_or_else(|| {
            crate::error::AppError::Manifest("missing best reducer state".to_owned())
        })?;
        let report = super::report::ReduceReport::best_so_far(
            &self.suite,
            &self.fixture_root,
            &self.work_dir,
            context,
            best_state,
            progress,
        );
        super::report::write_best(&self.work_dir, &report)
    }
}
