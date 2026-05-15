use std::io::{self, Read, Write};
use std::num::NonZeroUsize;
use std::process::ExitCode;

use fixture3_ddmin::{
    DdminGuarantee, DdminInput, DdminOptions, OracleOutcome, UnresolvedReason, ddmin,
};

type ProbeResult<T> = Result<T, String>;

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

fn run() -> ProbeResult<String> {
    let mut input = String::new();
    let _bytes_read = io::stdin().read_to_string(&mut input).map_err(|error| error.to_string())?;

    let mut output = String::new();
    for line in input.lines().filter(|line| !line.trim().is_empty()) {
        let request = Request::parse(line)?;
        let response = run_request(&request)?;
        output.push_str(&response);
        output.push('\n');
    }
    Ok(output)
}

fn run_request(request: &Request) -> ProbeResult<String> {
    let candidates: Vec<u8> = (0..request.candidate_count)
        .map(u8::try_from)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let options = DdminOptions::new(
        NonZeroUsize::new(request.initial_granularity)
            .ok_or_else(|| "initial granularity must be nonzero".to_owned())?,
        request.max_oracle_calls,
    );
    let mut oracle = |remaining: &[u8]| request.outcome_for(remaining);
    let result = ddmin(DdminInput::new(candidates, options), &mut oracle);
    let stats = result.stats();
    let guarantee = match result.guarantee() {
        DdminGuarantee::OneMinimalWithinCandidateSet => "complete".to_owned(),
        DdminGuarantee::Incomplete(reason) => format!("incomplete:{reason:?}"),
    };
    Ok(format!(
        "{} remaining={} removed={} guarantee={} calls={} interesting={} not_interesting={} unresolved={}",
        request.id,
        mask(result.remaining()),
        mask(result.removed()),
        guarantee,
        stats.oracle_calls(),
        stats.interesting_trials(),
        stats.not_interesting_trials(),
        stats.unresolved_trials()
    ))
}

#[derive(Debug)]
struct Request {
    id: String,
    candidate_count: usize,
    initial_granularity: usize,
    max_oracle_calls: Option<usize>,
    outcomes: Vec<u8>,
}

impl Request {
    fn parse(line: &str) -> ProbeResult<Self> {
        let mut parts = line.split_whitespace();
        let id = parts.next().ok_or_else(|| "missing id".to_owned())?;
        let candidate_count = parts.next().ok_or_else(|| "missing candidate count".to_owned())?;
        let initial_granularity =
            parts.next().ok_or_else(|| "missing initial granularity".to_owned())?;
        let max_oracle_calls = parts.next().ok_or_else(|| "missing max oracle calls".to_owned())?;
        let outcomes = parts.next().ok_or_else(|| "missing outcomes".to_owned())?;
        if parts.next().is_some() {
            return Err("probe request must have exactly 5 fields".to_owned());
        }
        Ok(Self {
            id: id.to_owned(),
            candidate_count: candidate_count.parse::<usize>().map_err(|error| error.to_string())?,
            initial_granularity: initial_granularity
                .parse::<usize>()
                .map_err(|error| error.to_string())?,
            max_oracle_calls: parse_max_calls(max_oracle_calls)?,
            outcomes: outcomes.as_bytes().to_vec(),
        })
    }

    fn outcome_for(&self, remaining: &[u8]) -> OracleOutcome {
        let outcome_index = mask(remaining);
        match self.outcomes.get(outcome_index).copied() {
            Some(b'I') => OracleOutcome::Interesting,
            Some(b'N') => OracleOutcome::NotInteresting,
            Some(b'U') => OracleOutcome::Unresolved(UnresolvedReason::InvalidCandidateSet),
            Some(_) | None => OracleOutcome::Unresolved(UnresolvedReason::OracleFailed),
        }
    }
}

fn parse_max_calls(value: &str) -> ProbeResult<Option<usize>> {
    if value == "none" {
        return Ok(None);
    }
    value.parse::<usize>().map(Some).map_err(|error| error.to_string())
}

fn mask(values: &[u8]) -> usize {
    values.iter().fold(0usize, |acc, value| acc | (1usize << usize::from(*value)))
}
