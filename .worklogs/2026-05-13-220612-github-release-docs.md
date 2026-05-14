Summary
- Updated README install and usage docs for the GitHub `v0.1.4` release.
- Clarified that the full agent-oriented guide is available in `fixture3 --help`.

Decisions made
- Documented direct GitHub tarball installation because `v0.1.4` is being released to GitHub before publishing a matching crates.io install stub.
- Kept the cargo-binstall section, but made it conditional on the matching crates.io stub being published.
- Added `check --all`, `status --all`, and aggregate exit-code behavior to the human README.

Key files for context
- `README.md`
- `.github/workflows/binary-release.yml`
- `.worklogs/2026-05-13-220029-all-suites-local-release.md`

Verification
- `scripts/verify-all.sh`
- GitHub release workflow for tag `v0.1.4`

Next steps
- Commit and push README docs.
- Push tag `v0.1.4`.
- Wait for all GitHub release assets to upload.
- Verify release assets exist for all supported targets.
