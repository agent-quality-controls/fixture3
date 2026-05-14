Summary
- Changed README install instructions so `cargo binstall fixture3` is the only documented install path.
- Removed direct GitHub release tarball install instructions from user-facing docs.

Decisions made
- Kept the crates.io `fixture3` package as an install stub for cargo-binstall metadata.
- Kept GitHub release archives as release artifacts because cargo-binstall resolves and downloads them through package metadata.
- Did not bump versions because local packages are already `0.1.5` and the matching GitHub release exists.

Key files for context
- `README.md`
- `.plans/2026-05-14-111202-binstall-only-install.md`
- `crates/fixture3-install/Cargo.toml`
- `crates/fixture3-install/src/main.rs`

Verification
- `scripts/verify-all.sh`
- README denylist search for direct release install commands.

Next steps
- Commit and push the docs change.
- Publish `fixture3 0.1.5` install stub to crates.io if it is not already published.
- Update GitHub release notes to show only the cargo-binstall install path.
