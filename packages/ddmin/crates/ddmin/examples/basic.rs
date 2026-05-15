use std::process::ExitCode;

use fixture3_ddmin::{DdminInput, DdminOptions, OracleOutcome, ddmin};

fn main() -> ExitCode {
    let input = DdminInput::new(vec![1_u8, 2, 3, 4], DdminOptions::default());
    let mut oracle = |remaining: &[u8]| {
        if remaining.contains(&2) && remaining.contains(&4) {
            OracleOutcome::Interesting
        } else {
            OracleOutcome::NotInteresting
        }
    };

    let output = ddmin(input, &mut oracle);
    if output.remaining() == [2, 4] { ExitCode::SUCCESS } else { ExitCode::from(1) }
}
