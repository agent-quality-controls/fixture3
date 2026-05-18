Summary
- Prepared `fixture3-ddmin` as a standalone public crate.
- Added crate README, rustdoc, examples, release metadata, manifest verifier coverage, and CI publish dry-run coverage.

Decisions made
- Kept `fixture3-ddmin` at `0.1.0` because this is its first crates.io publication.
- Published the algorithm crate directly instead of routing through the fixture3 CLI package.
- Added examples for normal reduction, budget exhaustion, unresolved oracle results, and file-path candidates because those are the smallest public usage surfaces agents need.
- Verified MSRV through the package manifest because `cargo msrv verify` cannot read `workspace.package.rust-version` from the virtual DDMin workspace root.

Key files for context
- `packages/ddmin/crates/ddmin/Cargo.toml`
- `packages/ddmin/crates/ddmin/README.md`
- `packages/ddmin/crates/ddmin/src/lib.rs`
- `packages/ddmin/crates/ddmin/src/types.rs`
- `packages/ddmin/crates/ddmin/src/algorithm.rs`
- `packages/ddmin/crates/ddmin/examples/basic.rs`
- `packages/ddmin/release-plz.toml`
- `packages/ddmin/cliff.toml`
- `packages/ddmin/scripts/verify-ddmin.py`
- `.github/workflows/ci.yml`

Verification
- `scripts/verify-ddmin.sh`
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --manifest-path packages/ddmin/Cargo.toml -p fixture3-ddmin`
- `cargo publish --dry-run --allow-dirty --manifest-path packages/ddmin/crates/ddmin/Cargo.toml`
- `python3 scripts/verify-rust-workspaces.py --fmt`
- `python3 scripts/verify-rust-workspaces.py --check`
- `python3 scripts/verify-rust-workspaces.py --clippy`
- `python3 scripts/verify-rust-workspaces.py --g3rs`
- `cargo msrv verify --manifest-path crates/ddmin/Cargo.toml -- cargo check --locked`
- `cargo publish --dry-run --allow-dirty --manifest-path apps/fixture3-install/crates/fixture3-install/Cargo.toml`
- `cargo run --manifest-path apps/fixtures/Cargo.toml -p fixture3-cli -- check --suite self --manifest fixture3.yaml`
- `g3rs validate repo`
- `scripts/verify-all.sh`
- `git diff --check`

Next steps
- Push `development`, open a PR into `main`, wait for CI, merge only after checks pass, then publish `fixture3-ddmin` from `main`.
