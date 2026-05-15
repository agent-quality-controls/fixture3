use std::io::{self, Write};
use std::num::NonZeroUsize;
use std::process::ExitCode;

use fixture3_ddmin::{
    DdminGuarantee, DdminInput, DdminOptions, DdminOutput, DdminStopReason, OracleOutcome,
    UnresolvedReason, ddmin,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Candidate {
    id: u8,
    label: &'static str,
}

fn main() -> ExitCode {
    match run() {
        Ok(output) => match io::stdout().write_all(output.as_bytes()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(_) => ExitCode::from(2),
        },
        Err(message) => match io::stderr().write_all(message.as_bytes()) {
            Ok(()) => ExitCode::from(1),
            Err(_) => ExitCode::from(2),
        },
    }
}

fn run() -> Result<String, String> {
    let cases = [
        ("baseline-interesting", baseline_interesting()),
        ("baseline-not-interesting", baseline_not_interesting()),
        ("empty-candidates", empty_candidates()),
        ("one-required-candidate", one_required_candidate()),
        ("two-required-candidates", two_required_candidates()),
        ("duplicate-labels-unique-ids", duplicate_labels_unique_ids()),
        ("order-preserving-subsequence", order_preserving_subsequence()),
        ("non-monotonic-oracle", non_monotonic_oracle()),
        ("unresolved-not-accepted", unresolved_not_accepted()),
        ("no-removable-candidate", no_removable_candidate()),
        ("max-oracle-calls", max_oracle_calls()),
    ];

    let mut output = String::new();
    for (name, result) in cases {
        result.map_err(|error| format!("{name}: {error}\n"))?;
        output.push_str("case: ");
        output.push_str(name);
        output.push_str(" PASS\n");
    }
    output.push_str("PASS\n");
    Ok(output)
}

fn baseline_interesting() -> Result<(), String> {
    let input = input(ids(&[1, 2, 3]));
    let mut oracle = requires(&[2]);
    let output = ddmin(input, &mut oracle);
    expect_remaining_ids(&output, &[2])?;
    expect_guarantee(&output, DdminGuarantee::OneMinimalWithinCandidateSet)
}

fn baseline_not_interesting() -> Result<(), String> {
    let input = input(ids(&[1, 2, 3]));
    let mut oracle = requires(&[9]);
    let output = ddmin(input, &mut oracle);
    expect_remaining_ids(&output, &[1, 2, 3])?;
    expect_guarantee(&output, DdminGuarantee::Incomplete(DdminStopReason::BaselineNotInteresting))
}

fn empty_candidates() -> Result<(), String> {
    let input = input(Vec::new());
    let mut oracle = requires(&[]);
    let output = ddmin(input, &mut oracle);
    expect_remaining_ids(&output, &[])?;
    expect_guarantee(&output, DdminGuarantee::OneMinimalWithinCandidateSet)
}

fn one_required_candidate() -> Result<(), String> {
    let input = input(ids(&[1]));
    let mut oracle = requires(&[1]);
    let output = ddmin(input, &mut oracle);
    expect_remaining_ids(&output, &[1])?;
    expect_guarantee(&output, DdminGuarantee::OneMinimalWithinCandidateSet)
}

fn two_required_candidates() -> Result<(), String> {
    let input = input(ids(&[1, 2, 3, 4]));
    let mut oracle = requires(&[1, 3]);
    let output = ddmin(input, &mut oracle);
    expect_remaining_ids(&output, &[1, 3])?;
    expect_guarantee(&output, DdminGuarantee::OneMinimalWithinCandidateSet)
}

fn duplicate_labels_unique_ids() -> Result<(), String> {
    let input = input(vec![
        Candidate { id: 1, label: "same" },
        Candidate { id: 2, label: "same" },
        Candidate { id: 3, label: "other" },
    ]);
    let mut oracle = requires(&[2]);
    let output = ddmin(input, &mut oracle);
    expect_remaining_ids(&output, &[2])?;
    expect_removed_ids(&output, &[1, 3])
}

fn order_preserving_subsequence() -> Result<(), String> {
    let input = input(ids(&[1, 2, 3, 4, 5]));
    let mut oracle = requires(&[2, 5]);
    let output = ddmin(input, &mut oracle);
    expect_remaining_ids(&output, &[2, 5])
}

fn non_monotonic_oracle() -> Result<(), String> {
    let input = input(ids(&[1, 2, 3, 4]));
    let mut oracle = |remaining: &[Candidate]| {
        let ids = candidate_ids(remaining);
        if ids == [1, 3] || ids == [1, 3, 4] || ids == [1, 2, 3] || ids == [1, 2, 3, 4] {
            OracleOutcome::Interesting
        } else {
            OracleOutcome::NotInteresting
        }
    };
    let output = ddmin(input, &mut oracle);
    expect_remaining_ids(&output, &[1, 3])
}

fn unresolved_not_accepted() -> Result<(), String> {
    let input = input(ids(&[1, 2, 3]));
    let mut oracle = |remaining: &[Candidate]| {
        let ids = candidate_ids(remaining);
        if ids == [2] {
            OracleOutcome::Unresolved(UnresolvedReason::InvalidCandidateSet)
        } else if ids.contains(&2) {
            OracleOutcome::Interesting
        } else {
            OracleOutcome::NotInteresting
        }
    };
    let output = ddmin(input, &mut oracle);
    expect_remaining_contains_id_with_len(&output, 2, 2)?;
    expect_unresolved_trials(&output, 1)
}

fn no_removable_candidate() -> Result<(), String> {
    let input = input(ids(&[1, 2, 3]));
    let mut oracle = |remaining: &[Candidate]| {
        if candidate_ids(remaining) == [1, 2, 3] {
            OracleOutcome::Interesting
        } else {
            OracleOutcome::NotInteresting
        }
    };
    let output = ddmin(input, &mut oracle);
    expect_remaining_ids(&output, &[1, 2, 3])
}

fn max_oracle_calls() -> Result<(), String> {
    let input = DdminInput::new(ids(&[1, 2, 3]), DdminOptions::new(NonZeroUsize::MIN, Some(1)));
    let mut oracle = requires(&[2]);
    let output = ddmin(input, &mut oracle);
    expect_guarantee(&output, DdminGuarantee::Incomplete(DdminStopReason::MaxOracleCallsReached))?;
    expect_oracle_calls(&output, 1)
}

fn input(candidates: Vec<Candidate>) -> DdminInput<Candidate> {
    DdminInput::new(candidates, DdminOptions::default())
}

fn ids(ids: &[u8]) -> Vec<Candidate> {
    ids.iter().copied().map(|id| Candidate { id, label: "id" }).collect()
}

fn requires(required: &'static [u8]) -> impl FnMut(&[Candidate]) -> OracleOutcome {
    move |remaining| {
        let remaining_ids = candidate_ids(remaining);
        if required.iter().all(|required_id| remaining_ids.contains(required_id)) {
            OracleOutcome::Interesting
        } else {
            OracleOutcome::NotInteresting
        }
    }
}

fn candidate_ids(candidates: &[Candidate]) -> Vec<u8> {
    candidates.iter().map(|candidate| candidate.id).collect()
}

fn expect_remaining_ids(output: &DdminOutput<Candidate>, expected: &[u8]) -> Result<(), String> {
    expect_ids("remaining", &candidate_ids(output.remaining()), expected)
}

fn expect_removed_ids(output: &DdminOutput<Candidate>, expected: &[u8]) -> Result<(), String> {
    expect_ids("removed", &candidate_ids(output.removed()), expected)
}

fn expect_remaining_contains_id_with_len(
    output: &DdminOutput<Candidate>,
    id: u8,
    len: usize,
) -> Result<(), String> {
    let actual = candidate_ids(output.remaining());
    if actual.len() == len && actual.contains(&id) {
        Ok(())
    } else {
        Err(format!("remaining expected length {len} and id {id}, got {actual:?}"))
    }
}

fn expect_ids(name: &str, actual: &[u8], expected: &[u8]) -> Result<(), String> {
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{name} expected {expected:?}, got {actual:?}"))
    }
}

fn expect_guarantee(
    output: &DdminOutput<Candidate>,
    expected: DdminGuarantee,
) -> Result<(), String> {
    if output.guarantee() == expected {
        Ok(())
    } else {
        Err(format!("guarantee expected {:?}, got {:?}", expected, output.guarantee()))
    }
}

fn expect_unresolved_trials(output: &DdminOutput<Candidate>, minimum: usize) -> Result<(), String> {
    if output.stats().unresolved_trials() >= minimum {
        Ok(())
    } else {
        Err(format!(
            "unresolved_trials expected at least {}, got {}",
            minimum,
            output.stats().unresolved_trials()
        ))
    }
}

fn expect_oracle_calls(output: &DdminOutput<Candidate>, expected: usize) -> Result<(), String> {
    if output.stats().oracle_calls() == expected {
        Ok(())
    } else {
        Err(format!("oracle_calls expected {}, got {}", expected, output.stats().oracle_calls()))
    }
}
