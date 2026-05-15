Summary
- Moved DDMin verification ownership into `packages/ddmin`.
- Replaced the Rust behavior example test harness with package-local JSON behavior data, Python verifiers, and a Rust probe crate.
- Kept the DDMin algorithm unchanged.

Decisions made
- Kept root `scripts/verify-ddmin.py` as a delegator so root verification still has one stable entry point.
- Made `scripts/rust-probe` a workspace member so Cargo and G3RS validate it with the DDMin workspace.
- Replaced the large Rust example harness with a small `examples/basic.rs` usage example.
- Kept generated reference cases derived from `behavior/cases/reference-matrix.json` instead of committing thousands of generated rows.

Key files for context
- `.plans/2026-05-15-172104-ddmin-verification-layout.md`
- `packages/ddmin/behavior/cases/contract.json`
- `packages/ddmin/behavior/cases/reference-matrix.json`
- `packages/ddmin/scripts/verify-ddmin.py`
- `packages/ddmin/scripts/run-contract-cases.py`
- `packages/ddmin/scripts/run-reference-differential.py`
- `packages/ddmin/scripts/rust-probe/src/main.rs`

Verification
- `scripts/verify-ddmin.sh`
- `python3 scripts/verify-rust-workspaces.py --fmt`
- `python3 scripts/verify-rust-workspaces.py --check`
- `python3 scripts/verify-rust-workspaces.py --clippy`
- `python3 scripts/verify-rust-workspaces.py --g3rs`
- `scripts/verify-all.sh`
- `git diff --check`

Next steps
- Use the package-local verifier as the place to add future reducer edge cases before connecting DDMin to fixture tree reduction.
