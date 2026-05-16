# Frontload Help And Default Reducer Budgets

## Goal

Make `fixture3 --help` and `fixture3 reduce --help` sufficient for an agent to operate the CLI without reading the README.

Set reducer defaults so copied-project reduction does not run unbounded:

- `dirs`: 200 oracle calls by default.
- `files`: 100 oracle calls by default.
- `dirs,files`: 200 calls for `dirs`, then 100 calls for `files`.

## Decisions

- Keep `--max-oracle-calls` as an explicit global override for compatibility.
- When `--max-oracle-calls` is absent, each reducer uses its own default budget.
- When `--max-oracle-calls` is present, it is a shared total cap across all reducers.
- Do not add new budget flags yet. One global override plus documented phase defaults is enough for the current CLI surface.
- Expand help text in `args.rs`; do not move help to README-only docs.

## Files

- `apps/fixtures/crates/fixture3/src/args.rs`
  - Replace top-level help with agent-complete usage guidance.
  - Replace reduce help with reducer concepts, budgets, manifest contract, report contract, and examples.
  - Update `--max-oracle-calls` help text to explain the global override.
- `apps/fixtures/crates/fixture3/src/reduce/run.rs`
  - Apply default per-reducer budgets when no global max is provided.
  - Preserve shared-budget behavior when `--max-oracle-calls` is present.
- `scripts/verify-reducer.py`
  - Verify help includes the reducer defaults and agent guidance.
  - Verify default fake-project reducer run reports `oracle_calls <= 300`.

## Verification

- `cargo fmt --manifest-path apps/fixtures/Cargo.toml --check`
- `cargo check --manifest-path apps/fixtures/Cargo.toml`
- `cargo clippy --manifest-path apps/fixtures/Cargo.toml --all-targets -- -D warnings`
- `python3 scripts/verify-reducer.py`
- `scripts/verify-all.sh`
- Install locally with `cargo install --path apps/fixtures/crates/fixture3 --locked --force`
- Confirm installed `fixture3 reduce --help` contains the full reducer guidance.
