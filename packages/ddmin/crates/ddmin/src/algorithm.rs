use crate::{
    DdminGuarantee, DdminInput, DdminOptions, DdminOracle, DdminOutput, DdminStats,
    DdminStopReason, OracleOutcome,
};

type PartitionRange = (usize, usize);

enum TrialResult<C> {
    Reduced(Vec<C>),
    NotReduced,
    LimitReached,
}

#[derive(Debug, Clone, Copy)]
enum TrialMode {
    Partition,
    Complement,
}

/// Minimize an ordered candidate list while preserving an oracle property.
///
/// The initial candidate list must be interesting. If it is not, the function
/// returns the original list with `DdminGuarantee::Incomplete`.
///
/// The returned `remaining` list is the smallest list accepted by this `DDMin`
/// run under the configured oracle-call budget. When the guarantee is complete,
/// the result is one-minimal within the input candidate set.
pub fn ddmin<C, O>(input: DdminInput<C>, oracle: &mut O) -> DdminOutput<C>
where
    C: Clone + Eq,
    O: DdminOracle<C>,
{
    let options = input.options();
    let original = input.into_candidates();
    let mut stats = DdminStats::default();

    let Some(baseline) = evaluate(&original, oracle, &options, &mut stats) else {
        return incomplete(original, Vec::new(), stats, DdminStopReason::MaxOracleCallsReached);
    };

    if !matches!(baseline, OracleOutcome::Interesting) {
        return incomplete(original, Vec::new(), stats, DdminStopReason::BaselineNotInteresting);
    }

    let mut remaining = original.clone();
    let mut granularity = options.initial_granularity().get().max(2);

    if remaining.len() <= 1 {
        return try_remove_only_candidate(&original, remaining, oracle, options, stats);
    }

    while remaining.len() >= 2 {
        granularity = granularity.clamp(2, remaining.len());
        let partitions = partition_ranges(remaining.len(), granularity);

        match try_ranges(
            &remaining,
            &partitions,
            TrialMode::Partition,
            oracle,
            &options,
            &mut stats,
        ) {
            TrialResult::Reduced(candidate) => {
                remaining = candidate;
                granularity = 2;
                continue;
            }
            TrialResult::NotReduced => {}
            TrialResult::LimitReached => {
                let removed = removed_from_original(&original, &remaining);
                return incomplete(
                    remaining,
                    removed,
                    stats,
                    DdminStopReason::MaxOracleCallsReached,
                );
            }
        }

        match try_ranges(
            &remaining,
            &partitions,
            TrialMode::Complement,
            oracle,
            &options,
            &mut stats,
        ) {
            TrialResult::Reduced(candidate) => {
                remaining = candidate;
                granularity = granularity.saturating_sub(1).max(2);
                continue;
            }
            TrialResult::NotReduced => {}
            TrialResult::LimitReached => {
                let removed = removed_from_original(&original, &remaining);
                return incomplete(
                    remaining,
                    removed,
                    stats,
                    DdminStopReason::MaxOracleCallsReached,
                );
            }
        }

        if granularity == remaining.len() {
            break;
        }
        granularity = granularity.saturating_mul(2).min(remaining.len());
    }

    if remaining.len() == 1 {
        return try_remove_only_candidate(&original, remaining, oracle, options, stats);
    }

    complete(&original, remaining, stats)
}

fn try_remove_only_candidate<C, O>(
    original: &[C],
    remaining: Vec<C>,
    oracle: &mut O,
    options: DdminOptions,
    mut stats: DdminStats,
) -> DdminOutput<C>
where
    C: Clone + Eq,
    O: DdminOracle<C>,
{
    match evaluate(&[], oracle, &options, &mut stats) {
        Some(OracleOutcome::Interesting) => complete(original, Vec::new(), stats),
        Some(OracleOutcome::NotInteresting | OracleOutcome::Unresolved(_)) => {
            complete(original, remaining, stats)
        }
        None => {
            let removed = removed_from_original(original, &remaining);
            incomplete(remaining, removed, stats, DdminStopReason::MaxOracleCallsReached)
        }
    }
}

fn try_ranges<C, O>(
    remaining: &[C],
    partitions: &[PartitionRange],
    mode: TrialMode,
    oracle: &mut O,
    options: &DdminOptions,
    stats: &mut DdminStats,
) -> TrialResult<C>
where
    C: Clone,
    O: DdminOracle<C>,
{
    for &(start, end) in partitions {
        let candidate = match mode {
            TrialMode::Partition => sublist(remaining, start, end),
            TrialMode::Complement => complement(remaining, start, end),
        };
        match evaluate(&candidate, oracle, options, stats) {
            Some(OracleOutcome::Interesting) => return TrialResult::Reduced(candidate),
            Some(OracleOutcome::NotInteresting | OracleOutcome::Unresolved(_)) => {}
            None => return TrialResult::LimitReached,
        }
    }
    TrialResult::NotReduced
}

fn evaluate<C, O>(
    candidate: &[C],
    oracle: &mut O,
    options: &DdminOptions,
    stats: &mut DdminStats,
) -> Option<OracleOutcome>
where
    O: DdminOracle<C>,
{
    if options.max_oracle_calls().is_some_and(|limit| stats.oracle_calls() >= limit) {
        return None;
    }

    let outcome = oracle.evaluate(candidate);
    stats.record(&outcome);
    Some(outcome)
}

fn partition_ranges(len: usize, count: usize) -> Vec<PartitionRange> {
    let mut ranges = Vec::with_capacity(count);
    let mut start = 0usize;
    for partition_index in 0..count {
        let remaining_len = len.saturating_sub(start);
        let remaining_partitions = count.saturating_sub(partition_index);
        let partition_len = remaining_len.div_ceil(remaining_partitions);
        let end = start.saturating_add(partition_len);
        ranges.push((start, end));
        start = end;
    }
    ranges
}

fn complement<C>(source: &[C], start: usize, end: usize) -> Vec<C>
where
    C: Clone,
{
    let mut result = Vec::with_capacity(source.len().saturating_sub(end.saturating_sub(start)));
    result.extend(source.iter().take(start).cloned());
    result.extend(source.iter().skip(end).cloned());
    result
}

fn sublist<C>(source: &[C], start: usize, end: usize) -> Vec<C>
where
    C: Clone,
{
    let len = end.saturating_sub(start);
    let mut result = Vec::with_capacity(len);
    result.extend(source.iter().skip(start).take(len).cloned());
    result
}

fn removed_from_original<C>(original: &[C], remaining: &[C]) -> Vec<C>
where
    C: Clone + Eq,
{
    let mut removed = Vec::new();
    let mut remaining_index = 0usize;
    for candidate in original {
        if remaining
            .get(remaining_index)
            .is_some_and(|remaining_candidate| remaining_candidate == candidate)
        {
            remaining_index = remaining_index.saturating_add(1);
        } else {
            removed.push(candidate.clone());
        }
    }
    removed
}

fn complete<C>(original: &[C], remaining: Vec<C>, stats: DdminStats) -> DdminOutput<C>
where
    C: Clone + Eq,
{
    let removed = removed_from_original(original, &remaining);
    DdminOutput::new(remaining, removed, stats, DdminGuarantee::OneMinimalWithinCandidateSet)
}

const fn incomplete<C>(
    remaining: Vec<C>,
    removed: Vec<C>,
    stats: DdminStats,
    reason: DdminStopReason,
) -> DdminOutput<C> {
    DdminOutput::new(remaining, removed, stats, DdminGuarantee::Incomplete(reason))
}
