use std::num::NonZeroUsize;

use fixture3_ddmin::{
    DdminGuarantee, DdminInput, DdminOptions, DdminStopReason, OracleOutcome, ddmin,
};

fn main() {
    let options = DdminOptions::new(NonZeroUsize::MIN, Some(1));
    let input = DdminInput::new(vec![1_u8, 2, 3, 4], options);
    let mut oracle = |remaining: &[u8]| {
        if remaining.contains(&2) {
            OracleOutcome::Interesting
        } else {
            OracleOutcome::NotInteresting
        }
    };

    let output = ddmin(input, &mut oracle);
    assert_eq!(
        output.guarantee(),
        DdminGuarantee::Incomplete(DdminStopReason::MaxOracleCallsReached),
        "DDMin should report an incomplete guarantee when the oracle-call budget is exhausted"
    );
    assert_eq!(output.stats().oracle_calls(), 1, "DDMin should stop at the configured budget");
}
