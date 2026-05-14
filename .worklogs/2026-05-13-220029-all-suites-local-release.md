Summary
- Implemented `fixture3 check --all` and `fixture3 status --all`.
- Added self fixtures for all-match, mixed-diff, error precedence, status-all, and invalid `--suite` / `--all` combinations.
- Bumped local package versions to `0.1.4`.

Decisions made
- Kept `check --suite <name>` as the single-suite path and made `check` require either `--suite` or `--all`.
- Kept bare `status` as an all-suite compatibility alias while adding explicit `status --all`.
- Reused the existing single-suite `run_check` path for aggregate checks so command execution, normalization, storage, and diff logic stay unified.
- Aggregated exit codes as `2` for any suite error, `1` for any suite diff without errors, and `0` only when every suite matches.
- Did not create a GitHub tag, GitHub release, or crates.io publish for `0.1.4`.

Key files for context
- `.plans/2026-05-13-215321-all-suites.md`
- `.plans/2026-05-13-215321-all-suites.md.manifest.toml`
- `crates/fixture3/src/args.rs`
- `crates/fixture3/src/app.rs`
- `scripts/self-check-harness.py`
- `behavior/fixtures/self/cases/check-all-match/fixture3.yaml`
- `behavior/fixtures/self/cases/check-all-mismatch/fixture3.yaml`
- `behavior/fixtures/self/cases/check-all-error/fixture3.yaml`
- `behavior/fixtures/self/cases/status-all/fixture3.yaml`
- `behavior/golden/self/approved.normalized.json`
- `behavior/golden/self/approved.meta.json`

Verification
- `cargo check`
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `python3 scripts/self-check-harness.py behavior/fixtures/self/cases/*/fixture3.yaml`
- `scripts/verify-all.sh`

Next steps
- Commit the implementation.
- Install `0.1.4` locally with `cargo install --path crates/fixture3 --force --locked`.
- Verify `/Users/tartakovsky/.cargo/bin/fixture3 --version` reports `fixture3 0.1.4`.
