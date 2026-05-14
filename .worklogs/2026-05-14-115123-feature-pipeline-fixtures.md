Summary
- Migrated repository approval storage from old approved-output paths to `behavior/approved` paths.
- Added feature-aware suite management: suite tags, feature groups, `--tag`, `--feature`, `explain`, `doctor`, `new suite`, and JSON output for check, diff, and status.
- Added self fixtures and a feature-pipeline verifier manifest for the new contract.

Decisions made
- Kept fixtures as inputs and approvals as reviewed outputs; feature semantics stay project-owned.
- Kept `approved_dir`, `approved.normalized.json`, and `approved.meta.json` as the durable output contract.
- Made `new suite` create files and print a manifest block instead of editing YAML.
- Bumped packages to `0.1.6` because `0.1.5` is already published.

Key files for context
- `.plans/2026-05-14-113743-feature-pipeline-fixtures.md`
- `.plans/2026-05-14-113743-feature-pipeline-fixtures.md.manifest.toml`
- `crates/fixture3/src/app.rs`
- `crates/fixture3/src/args.rs`
- `crates/fixture3/src/manifest.rs`
- `crates/fixture3/src/selection.rs`
- `crates/fixture3/src/doctor.rs`
- `crates/fixture3/src/scaffold.rs`
- `scripts/self-check-harness.py`
- `scripts/verify-feature-pipeline.py`
- `fixture3.yaml`
- `behavior/approved/self/approved.normalized.json`

Verification
- `scripts/verify-all.sh`
- `fixture3 explain --suite self --manifest fixture3.yaml`
- `fixture3 doctor --manifest fixture3.yaml`
- `fixture3 status --feature self --manifest fixture3.yaml --json`
- `fixture3 check --tag fixture-pipeline --manifest fixture3.yaml --json`
- `cargo publish -p fixture3 --dry-run --allow-dirty`

Next steps
- Commit and push the implementation.
- Install `fixture3 0.1.6` locally from `crates/fixture3`.
- Release `0.1.6` only after deciding the new CLI surface is stable enough to publish.
