Summary
- Added `examples/fake-project` as an external-style project that uses fixture3 features, tags, suites, approvals, and changes.
- Added `scripts/verify-fake-project.sh` to run the feature-pipeline workflow against a disposable copy.
- Wired the fake-project verifier into `scripts/verify-all.sh`.

Decisions made
- Kept mutating commands on `.fixture3/fake-project-run` so `approve` and `new suite` are verified without changing the committed example.
- Used a tiny local JSON-producing script for fake app behavior, with no new dependencies.
- Added a manifest for the fake-project verifier so expected files are mechanically checked.

Key files for context
- `examples/fake-project/fixture3.yaml`
- `examples/fake-project/scripts/fake-app.sh`
- `scripts/verify-fake-project.sh`
- `.plans/2026-05-14-122624-fake-project-verification.md`
- `.plans/2026-05-14-122624-fake-project-verification.md.manifest.toml`
- `README.md`

Verification
- `scripts/verify-fake-project.sh`
- `cargo fmt --check`
- `scripts/verify-all.sh`

Next steps
- Commit and push the fake-project verifier.
