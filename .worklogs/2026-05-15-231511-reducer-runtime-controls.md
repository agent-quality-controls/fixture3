Summary
- Added reducer runtime controls: `--max-oracle-calls`, best-so-far reports, and one reusable trial directory.
- Updated the reducer verifier to prove limited runs, best report output, and removal of retained trial history.

Decisions made
- Passed `--max-oracle-calls` into `fixture3-ddmin` instead of stopping outside the algorithm.
- Wrote best-so-far reports from the oracle because only the oracle sees successful trials before DDMin returns.
- Reused `<work-dir>/trial-current` and removed stale `<work-dir>/trials` at run start to avoid unbounded disk growth.

Key files for context
- `.plans/2026-05-15-225212-reducer-runtime-controls.md`
- `.plans/2026-05-15-225212-reducer-runtime-controls.md.manifest.toml`
- `apps/fixtures/crates/fixture3/src/reduce/run.rs`
- `apps/fixtures/crates/fixture3/src/reduce/oracle.rs`
- `apps/fixtures/crates/fixture3/src/reduce/report.rs`
- `apps/fixtures/crates/fixture3/src/reduce/trial_tree.rs`
- `scripts/verify-reducer.py`

Verification
- `python3 scripts/verify-reducer.py`
- `python3 scripts/verify-rust-workspaces.py --fmt`
- `python3 scripts/verify-rust-workspaces.py --check`
- `python3 scripts/verify-rust-workspaces.py --clippy`
- `python3 scripts/verify-rust-workspaces.py --g3rs`
- `python3 scripts/verify-manifest.py cli`
- `scripts/verify-all.sh`
- `git diff --check`

Next steps
- Use `--max-oracle-calls` in the Guardrail3 L45 probe before deciding whether to keep a permanent copied fixture.
