use std::path::PathBuf;

use fixture3_ddmin::{OracleOutcome, UnresolvedReason};

use super::candidate::FileCandidate;

#[derive(Debug)]
pub(crate) struct ReduceOracle {
    suite: String,
    manifest_path: PathBuf,
    fixture_root: PathBuf,
    work_dir: PathBuf,
    calls: usize,
}

impl ReduceOracle {
    pub(crate) const fn new(
        suite: String,
        manifest_path: PathBuf,
        fixture_root: PathBuf,
        work_dir: PathBuf,
    ) -> Self {
        Self { suite, manifest_path, fixture_root, work_dir, calls: 0 }
    }

    pub(crate) const fn calls(&self) -> usize {
        self.calls
    }
}

impl fixture3_ddmin::DdminOracle<FileCandidate> for ReduceOracle {
    fn evaluate(&mut self, remaining: &[FileCandidate]) -> OracleOutcome {
        self.calls = self.calls.saturating_add(1);
        let Ok(trial) =
            super::trial_tree::create(self.calls, &self.fixture_root, &self.work_dir, remaining)
        else {
            return OracleOutcome::Unresolved(UnresolvedReason::OracleFailed);
        };
        if super::rewrite::write_trial_manifest(&self.manifest_path, &self.suite, &trial, remaining)
            .is_err()
        {
            return OracleOutcome::Unresolved(UnresolvedReason::OracleFailed);
        }

        match crate::app::run_check(&self.suite, trial.manifest_path()) {
            Ok(result) if result.report.exit_code() == 0 => OracleOutcome::Interesting,
            Ok(_) => OracleOutcome::NotInteresting,
            Err(_) => OracleOutcome::Unresolved(UnresolvedReason::OracleFailed),
        }
    }
}
