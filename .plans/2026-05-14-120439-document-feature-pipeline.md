Goal
- Document feature-aware fixture pipeline behavior in both `fixture3 --help` and `README.md`.
- Cover tags, features, selectors, JSON output, explain, doctor, new suite, approval files, and agent workflow.

Approach
- Expand top-level CLI help so an agent can learn the full model without walking the help tree.
- Expand subcommand help for `check`, `status`, `diff`, `explain`, `doctor`, and `new`.
- Update README with a clear feature pipeline section, manifest schema, workflows, command reference, and JSON examples.
- Update help verifier manifest expectations where new help strings are contractually important.

Files to modify
- `README.md`
- `crates/fixture3/src/args.rs`
- `.plans/2026-05-13-150929-fixture3-architecture.md.manifest.toml`
- `.plans/2026-05-14-113743-feature-pipeline-fixtures.md.manifest.toml`

Verification
- `scripts/verify-all.sh`
- `fixture3 --help`
- `fixture3 check --help`
- `fixture3 explain --help`
- `fixture3 doctor --help`
- `fixture3 new --help`
