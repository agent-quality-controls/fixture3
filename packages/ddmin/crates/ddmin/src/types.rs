use std::num::NonZeroUsize;

/// Input for one `DDMin` reduction run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DdminInput<C> {
    candidates: Vec<C>,
    options: DdminOptions,
}

impl<C> DdminInput<C> {
    /// Create a `DDMin` input from ordered candidates and execution options.
    #[must_use]
    pub const fn new(candidates: Vec<C>, options: DdminOptions) -> Self {
        Self { candidates, options }
    }

    pub(crate) const fn options(&self) -> DdminOptions {
        self.options
    }

    pub(crate) fn into_candidates(self) -> Vec<C> {
        self.candidates
    }
}

/// Execution options for `DDMin`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DdminOptions {
    initial_granularity: NonZeroUsize,
    max_oracle_calls: Option<usize>,
}

impl DdminOptions {
    /// Create execution options.
    ///
    /// `initial_granularity` controls the first split count. Values below two are accepted
    /// and normalized by the algorithm. `max_oracle_calls` stops the run after that many
    /// oracle evaluations.
    #[must_use]
    pub const fn new(initial_granularity: NonZeroUsize, max_oracle_calls: Option<usize>) -> Self {
        Self { initial_granularity, max_oracle_calls }
    }

    /// Requested initial split count.
    #[must_use]
    pub const fn initial_granularity(self) -> NonZeroUsize {
        self.initial_granularity
    }

    /// Optional maximum number of oracle evaluations for one run.
    #[must_use]
    pub const fn max_oracle_calls(self) -> Option<usize> {
        self.max_oracle_calls
    }
}

impl Default for DdminOptions {
    fn default() -> Self {
        Self { initial_granularity: NonZeroUsize::MIN, max_oracle_calls: None }
    }
}

/// Result of evaluating one candidate list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OracleOutcome {
    /// The candidate list still preserves the target property.
    Interesting,
    /// The candidate list does not preserve the target property.
    NotInteresting,
    /// The oracle could not safely decide for this candidate list.
    Unresolved(UnresolvedReason),
}

/// Reason an oracle could not evaluate a candidate list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnresolvedReason {
    /// The oracle exceeded its time budget.
    Timeout,
    /// The caller could not build the trial state for this candidate list.
    MaterializationFailed,
    /// The oracle command, callback, or evaluation code failed.
    OracleFailed,
    /// The candidate list is invalid for the caller's domain.
    InvalidCandidateSet,
    /// The oracle detected nondeterministic behavior.
    NonDeterministic,
}

/// Callback boundary between `DDMin` and the system being reduced.
pub trait DdminOracle<C> {
    /// Evaluate a candidate list.
    ///
    /// The slice contains the candidates that remain in this trial.
    fn evaluate(&mut self, remaining: &[C]) -> OracleOutcome;
}

impl<C, F> DdminOracle<C> for F
where
    F: FnMut(&[C]) -> OracleOutcome,
{
    fn evaluate(&mut self, remaining: &[C]) -> OracleOutcome {
        self(remaining)
    }
}

/// Output from one `DDMin` reduction run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DdminOutput<C> {
    remaining: Vec<C>,
    removed: Vec<C>,
    stats: DdminStats,
    guarantee: DdminGuarantee,
}

impl<C> DdminOutput<C> {
    pub(crate) const fn new(
        remaining: Vec<C>,
        removed: Vec<C>,
        stats: DdminStats,
        guarantee: DdminGuarantee,
    ) -> Self {
        Self { remaining, removed, stats, guarantee }
    }

    /// Candidates that remain after reduction.
    #[must_use]
    pub fn remaining(&self) -> &[C] {
        &self.remaining
    }

    /// Candidates removed from the original input.
    #[must_use]
    pub fn removed(&self) -> &[C] {
        &self.removed
    }

    /// Oracle-call accounting for this run.
    #[must_use]
    pub const fn stats(&self) -> DdminStats {
        self.stats
    }

    /// Completion guarantee for this run.
    #[must_use]
    pub const fn guarantee(&self) -> DdminGuarantee {
        self.guarantee
    }
}

/// Oracle accounting for one `DDMin` run.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DdminStats {
    oracle_calls: usize,
    interesting_trials: usize,
    not_interesting_trials: usize,
    unresolved_trials: usize,
}

impl DdminStats {
    pub(crate) const fn record(&mut self, outcome: &OracleOutcome) {
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
    }

    /// Total oracle evaluations performed.
    #[must_use]
    pub const fn oracle_calls(self) -> usize {
        self.oracle_calls
    }

    /// Oracle evaluations that returned `Interesting`.
    #[must_use]
    pub const fn interesting_trials(self) -> usize {
        self.interesting_trials
    }

    /// Oracle evaluations that returned `NotInteresting`.
    #[must_use]
    pub const fn not_interesting_trials(self) -> usize {
        self.not_interesting_trials
    }

    /// Oracle evaluations that returned `Unresolved`.
    #[must_use]
    pub const fn unresolved_trials(self) -> usize {
        self.unresolved_trials
    }
}

/// Guarantee reached by one `DDMin` run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DdminGuarantee {
    /// The run completed and the result is one-minimal within the input candidate set.
    OneMinimalWithinCandidateSet,
    /// The run stopped before reaching the normal completion guarantee.
    Incomplete(DdminStopReason),
}

/// Reason `DDMin` stopped before reaching its normal guarantee.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DdminStopReason {
    /// The configured oracle-call limit was reached.
    MaxOracleCallsReached,
    /// The original candidate list was not interesting.
    BaselineNotInteresting,
}
