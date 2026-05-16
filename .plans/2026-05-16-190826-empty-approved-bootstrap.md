# Empty approved bootstrap and approve comments

Goal
- Missing approved output is not a tool error.
- On first check, fixture3 creates approved.normalized.json with canonical empty JSON: `{}`.
- The normal compare path then runs: `{}` versus received output.
- `approve --comment <text>` records optional approval metadata.
- Remove `--change` as the public name.

Approach
- Update behavior harness before fixing code so the current bug is visible.
- Change storage write path to create missing approved output as `{}` and continue.
- Rename approval metadata from `change_path` to `comment`.
- Rename CLI approve argument from `--change` to `--comment`.
- Update help, README, self fixtures, and verifier manifest text.
- Regenerate approved self output.
- Verify with fixture3 behavior suites and static checks.

Files
- apps/fixtures/crates/fixture3/src/storage.rs
- apps/fixtures/crates/fixture3/src/metadata.rs
- apps/fixtures/crates/fixture3/src/app.rs
- apps/fixtures/crates/fixture3/src/args.rs
- scripts/self-check-harness.py
- behavior/fixtures/self/cases/*
- README.md
- .plans/2026-05-13-150929-fixture3-architecture.md.manifest.toml
