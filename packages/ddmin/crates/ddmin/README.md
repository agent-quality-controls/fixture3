# fixture3-ddmin

`fixture3-ddmin` is a small Rust library for running the DDMin reduction algorithm on an ordered list of candidates.

It is useful when you have a reproducible property and want to remove as many inputs as possible while that property still holds. The library does not know what your candidates mean. Your oracle decides whether a candidate list is still interesting.

## Install

```toml
[dependencies]
fixture3-ddmin = "0.1"
```

## Core Model

DDMin needs four things:

- `candidates`: the ordered list to reduce.
- `oracle`: a callback that evaluates the remaining candidates.
- `OracleOutcome::Interesting`: the property still holds.
- `OracleOutcome::NotInteresting`: the property no longer holds.

The algorithm repeatedly tries smaller candidate lists. When the oracle says a smaller list is interesting, that list becomes the new baseline.

The output contains:

- `remaining`: candidates still needed.
- `removed`: candidates removed from the original input.
- `stats`: oracle call counts.
- `guarantee`: whether the run completed or stopped early.

## Minimal Example

```rust
use fixture3_ddmin::{DdminInput, DdminOptions, OracleOutcome, ddmin};

let input = DdminInput::new(vec![1_u8, 2, 3, 4], DdminOptions::default());
let mut oracle = |remaining: &[u8]| {
    if remaining.contains(&2) && remaining.contains(&4) {
        OracleOutcome::Interesting
    } else {
        OracleOutcome::NotInteresting
    }
};

let output = ddmin(input, &mut oracle);
assert_eq!(output.remaining(), &[2, 4]);
assert_eq!(output.removed(), &[1, 3]);
```

## What "Interesting" Means

"Interesting" is the property you want to preserve.

Examples:

- A compiler still fails with the same diagnostic.
- A command still prints the same JSON row.
- A fixture still triggers a rule.
- A generated API shape still contains a target item.
- A parser still returns the same reduced AST node.

The oracle should be deterministic. If the same candidate list sometimes returns different outcomes, DDMin cannot make a reliable guarantee.

## Candidate Lists

Candidates can be any `Clone + Eq` type.

Good candidates:

- file paths
- line IDs
- AST node IDs
- feature names
- config keys
- input records

The algorithm treats candidates as an ordered list. It does not inspect files, parse code, run commands, or mutate state. Put those operations inside your oracle.

## Oracle Outcomes

Use `OracleOutcome::Interesting` when the current candidate list preserves the target property.

Use `OracleOutcome::NotInteresting` when it does not.

Use `OracleOutcome::Unresolved(reason)` when the oracle cannot safely decide. Unresolved trials are treated like rejected removals: the algorithm keeps searching, but it will not accept that candidate list.

Unresolved reasons:

- `Timeout`
- `MaterializationFailed`
- `OracleFailed`
- `InvalidCandidateSet`
- `NonDeterministic`

These reasons are reported in stats only as unresolved trial counts. They do not change the reduction rule.

## Guarantees

`DdminGuarantee::OneMinimalWithinCandidateSet` means the algorithm completed and found a one-minimal result within the candidate set it was given.

One-minimal means: removing any single remaining candidate would make the oracle stop reporting `Interesting`.

It does not mean:

- globally smallest possible result
- shortest possible file tree
- best semantic fixture design
- minimum under a different candidate model

`DdminGuarantee::Incomplete(DdminStopReason::MaxOracleCallsReached)` means the configured oracle-call limit stopped the run before completion.

`DdminGuarantee::Incomplete(DdminStopReason::BaselineNotInteresting)` means the original input was not interesting. DDMin cannot reduce an input that does not satisfy the target property at the start.

## Budgeted Runs

Use `DdminOptions::new(initial_granularity, Some(max_calls))` to cap oracle calls.

```rust
use std::num::NonZeroUsize;
use fixture3_ddmin::{DdminInput, DdminOptions};

let options = DdminOptions::new(NonZeroUsize::new(2).unwrap(), Some(100));
let input = DdminInput::new(vec!["a", "b", "c"], options);
```

When the budget is reached, the output still contains the best accepted `remaining` list found so far, but the guarantee is incomplete.

## File Tree Example

The library does not delete files. It reduces a list. A file-tree reducer should:

1. Walk the fixture tree.
2. Build candidates such as relative file paths or directory paths.
3. For each oracle call, create a temporary copy with only the candidate paths.
4. Run the project command against the temporary copy.
5. Return `Interesting` only when the behavior contract still holds.

That keeps filesystem mutation outside the algorithm.

## When Not To Use This

Do not use DDMin when:

- the oracle is nondeterministic
- the baseline input is not interesting
- candidate order is meaningless and should be modeled differently
- you need a proof of global minimality
- each oracle run is unsafe or has uncontained side effects

## Verification

This crate is checked against:

- contract cases
- differential cases against a reference implementation
- invariant checks over generated oracle matrices
- Rust workspace checks
- G3RS validation in this repository

The verification entry point in the repository is:

```bash
scripts/verify-ddmin.sh
```
