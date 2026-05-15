Summary
- Corrected the DDMin plan from an internal fixture3 module to a standalone workspace crate under `packages/ddmin`.
- Added crate metadata, dual-license rationale, and an explicit oracle callback contract.

Decisions made
- Chose package name `fixture3-ddmin`.
- Kept `MIT OR Apache-2.0` to match the Rust ecosystem and current workspace metadata.
- Made `DdminOracle` the callback boundary and added a blanket implementation for `FnMut(&[C]) -> OracleOutcome`.
- Kept filesystem materialization and fixture contract execution out of the algorithm crate.

Key files for context
- `.plans/2026-05-15-145518-ddmin-package.md`
- `Cargo.toml`

Verification
- Read root `Cargo.toml`.
- `git diff --check`.

Next steps
- Implement `packages/ddmin` from the standalone plan.
- Add behavior verification without Rust unit tests.
