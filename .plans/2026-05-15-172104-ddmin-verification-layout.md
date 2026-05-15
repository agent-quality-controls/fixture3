# DDMin Verification Layout Plan

## Goal

Move DDMin verification into the DDMin workspace without changing the DDMin algorithm.

The repo-root verifier remains only a delegator.

## Approach

- Replace `packages/ddmin/crates/ddmin/examples/behavior.rs` with a small human-facing `examples/basic.rs`.
- Add package-local behavior case data under `packages/ddmin/behavior/cases`.
- Add a package-local Rust probe under `packages/ddmin/scripts/rust-probe`.
- Add package-local Python verifiers under `packages/ddmin/scripts`.
- Update the DDMin manifest to track the new files and behavior cases.
- Update root `scripts/verify-ddmin.py` to delegate to `packages/ddmin/scripts/verify-ddmin.py`.

## Key Decisions

- Keep the Rust algorithm files unchanged.
- Keep verification outside Rust unit tests, integration tests, doc tests, and `tests/`.
- Keep reference comparison generated from committed verifier config instead of committing thousands of generated cases.
- Use the Zeller-compatible reference policy with the same partition ordering as the Rust implementation so exact comparisons are meaningful.

## Files To Modify

- `.plans/2026-05-15-145518-ddmin-package.md.manifest.toml`
- `scripts/verify-ddmin.py`
- `packages/ddmin/crates/ddmin/examples/behavior.rs`
- `packages/ddmin/crates/ddmin/examples/basic.rs`
- `packages/ddmin/behavior/cases/contract.json`
- `packages/ddmin/behavior/cases/reference-matrix.json`
- `packages/ddmin/scripts/verify-ddmin.py`
- `packages/ddmin/scripts/run-contract-cases.py`
- `packages/ddmin/scripts/run-reference-differential.py`
- `packages/ddmin/scripts/reference-ddmin.py`
- `packages/ddmin/scripts/rust-probe/Cargo.toml`
- `packages/ddmin/scripts/rust-probe/src/main.rs`

## Done State

- `scripts/verify-ddmin.sh` passes.
- `scripts/verify-all.sh` passes.
- `g3rs validate --path packages/ddmin --rules-only` passes.
- `git diff --check` passes.
