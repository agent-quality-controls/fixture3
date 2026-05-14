Summary
- Expanded `README.md` with the full fixture3 model, feature pipeline manifest shape, JSON output, command reference, failure modes, and repository verification.
- Expanded top-level and subcommand help for tags, features, selectors, JSON output, `explain`, `doctor`, and `new suite`.
- Updated manifest verifier expectations so the help contract covers the new documentation.

Decisions made
- Kept `fixture3 --help` as the agent-facing guide instead of relying on subcommand traversal.
- Kept README organized around install, model, manifest, files, workflow, agent output, commands, and failure behavior.
- Verified help text through existing manifest checks rather than adding Rust tests.

Key files for context
- `README.md`
- `crates/fixture3/src/args.rs`
- `.plans/2026-05-13-150929-fixture3-architecture.md.manifest.toml`
- `.plans/2026-05-14-113743-feature-pipeline-fixtures.md.manifest.toml`
- `.plans/2026-05-14-120439-document-feature-pipeline.md`

Verification
- `cargo fmt --check`
- `scripts/verify-all.sh`
- `fixture3 --help`
- `fixture3 explain --help`
- `fixture3 new --help`

Next steps
- Commit and push the documentation update.
