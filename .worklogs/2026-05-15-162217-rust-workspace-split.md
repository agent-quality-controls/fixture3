Summary
- Split the repository into three independent Rust workspaces: `apps/fixtures`, `apps/fixture3-install`, and `packages/ddmin`.
- Moved the CLI, cargo-install stub, and DDMin crate under those workspaces and removed the root Rust workspace marker pair.
- Updated manifests, verifiers, workflows, G3RS config, and fixture harness paths to use the new workspace roots.

Decisions made
- Kept only `[checks] test = false` in each workspace G3RS config; all other rule families remain active.
- Kept the root as a coordination root for docs, plans, scripts, behavior fixtures, and GitHub workflows.
- Added package-local lockfiles, toolchain files, lint config, and release material because `g3rs validate --path <workspace>` treats each workspace as its own adopted unit.
- Fixed the JSON parsing lint by marking the existing JSON parser functions as the explicit boundary instead of weakening the lint policy.

Key files for context
- `.plans/2026-05-15-160906-rust-workspace-split.md`
- `.plans/2026-05-15-160906-rust-workspace-split.md.manifest.toml`
- `scripts/verify-rust-workspaces.py`
- `apps/fixtures/Cargo.toml`
- `apps/fixture3-install/Cargo.toml`
- `packages/ddmin/Cargo.toml`
- `.githooks/pre-commit`

Verification
- `python3 scripts/verify-rust-workspaces.py`
- `g3rs validate-repo`
- `python3 scripts/verify-rust-workspaces.py --g3rs`
- `python3 scripts/verify-rust-workspaces.py --fmt`
- `python3 scripts/verify-rust-workspaces.py --check`
- `python3 scripts/verify-rust-workspaces.py --clippy`
- `scripts/verify-all.sh`
- `cargo publish --dry-run --allow-dirty --manifest-path apps/fixture3-install/crates/fixture3-install/Cargo.toml`
- `git diff --check`

Next steps
- Publish from a clean commit when the install stub is ready for the next crates.io version.
