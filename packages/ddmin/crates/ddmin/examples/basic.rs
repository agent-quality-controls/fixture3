use fixture3_ddmin::{DdminInput, DdminOptions, OracleOutcome, ddmin};

fn main() {
    let input = DdminInput::new(vec![1_u8, 2, 3, 4], DdminOptions::default());
    let mut oracle = |remaining: &[u8]| {
        if remaining.contains(&2) && remaining.contains(&4) {
            OracleOutcome::Interesting
        } else {
            OracleOutcome::NotInteresting
        }
    };

    let output = ddmin(input, &mut oracle);
    assert_eq!(output.remaining(), &[2, 4], "DDMin should keep the required values");
    assert_eq!(output.removed(), &[1, 3], "DDMin should remove the irrelevant values");
}
