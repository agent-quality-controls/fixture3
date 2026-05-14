Goal
- Make `cargo binstall fixture3` the only documented install path.
- Remove direct GitHub release tarball install instructions from README and release notes.
- Publish the matching crates.io install stub if `fixture3 0.1.5` is not already published.

Approach
- Rewrite the top README install block to show `cargo install cargo-binstall` and `cargo binstall fixture3`.
- Keep GitHub release assets as implementation detail for cargo-binstall, not a user-facing install command.
- Update release notes for `v0.1.5` with the same install path.
- Verify the repo and dry-run the install stub package.

Key decisions
- Do not bump package versions unless crates.io already contains `fixture3 0.1.5`.
- Do not remove GitHub binary release assets because cargo-binstall needs those archives.

Files to modify
- `README.md`
- `.plans/2026-05-14-111202-binstall-only-install.md`
- `.worklogs/<timestamp>-binstall-only-install.md`
