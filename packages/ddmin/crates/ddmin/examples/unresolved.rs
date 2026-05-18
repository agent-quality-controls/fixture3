use fixture3_ddmin::{DdminInput, DdminOptions, OracleOutcome, UnresolvedReason, ddmin};

fn main() {
    let input = DdminInput::new(vec!["core", "flaky", "noise"], DdminOptions::default());
    let mut oracle = |remaining: &[&str]| {
        if remaining.contains(&"flaky") && remaining.len() == 1 {
            OracleOutcome::Unresolved(UnresolvedReason::NonDeterministic)
        } else if remaining.contains(&"core") {
            OracleOutcome::Interesting
        } else {
            OracleOutcome::NotInteresting
        }
    };

    let output = ddmin(input, &mut oracle);
    assert_eq!(output.remaining(), &["core"], "DDMin should keep the stable interesting value");
    assert_eq!(
        output.stats().unresolved_trials(),
        1,
        "DDMin should count unresolved oracle outcomes"
    );
}
