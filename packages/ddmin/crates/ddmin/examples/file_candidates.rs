use std::process::ExitCode;

use fixture3_ddmin::{DdminInput, DdminOptions, OracleOutcome, ddmin};

#[derive(Clone, Debug, PartialEq, Eq)]
struct FileCandidate {
    path: &'static str,
    required: bool,
}

fn main() -> ExitCode {
    let input = DdminInput::new(
        vec![
            FileCandidate { path: "fixture/src/lib.rs", required: true },
            FileCandidate { path: "fixture/README.md", required: false },
            FileCandidate { path: "fixture/noise.log", required: false },
        ],
        DdminOptions::default(),
    );
    let mut oracle = |remaining: &[FileCandidate]| {
        if remaining.iter().any(|candidate| candidate.required) {
            OracleOutcome::Interesting
        } else {
            OracleOutcome::NotInteresting
        }
    };

    let output = ddmin(input, &mut oracle);
    let [remaining] = output.remaining() else {
        return ExitCode::from(1);
    };
    if remaining.path == "fixture/src/lib.rs" { ExitCode::SUCCESS } else { ExitCode::from(1) }
}
