Summary
- Implemented `fixture3-ddmin` as a standalone sequential DDMin crate under `packages/ddmin`.
- Added a manifest and verifier for the DDMin package plan.
- Wired the DDMin verifier into `scripts/verify-all.sh`.

Decisions made
- Kept the algorithm sequential.
- Split the crate into facade `lib.rs`, `algorithm.rs`, and `types.rs` because G3RS requires facade-only library roots.
- Used private public-API fields with constructors and getters because G3RS rejects public named fields.
- Added an explicit `algorithm` feature because G3RS requires facade exports to be feature-gated.
- Verified behavior through a Cargo example plus `scripts/verify-ddmin.py`, not Rust tests.

Key files for context
- `.plans/2026-05-15-145518-ddmin-package.md`
- `.plans/2026-05-15-145518-ddmin-package.md.manifest.toml`
- `packages/ddmin/src/lib.rs`
- `packages/ddmin/src/algorithm.rs`
- `packages/ddmin/src/types.rs`
- `packages/ddmin/examples/behavior.rs`
- `scripts/verify-ddmin.py`
- `scripts/verify-ddmin.sh`

Verification
- `scripts/verify-ddmin.sh`
- `cargo fmt --check`
- `cargo check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `g3rs validate --path . --rules-only`
- `scripts/verify-all.sh`

Next steps
- Use `fixture3-ddmin` from the fixture reducer only after the fixture reducer plan is rewritten from the remaining research tasks.
