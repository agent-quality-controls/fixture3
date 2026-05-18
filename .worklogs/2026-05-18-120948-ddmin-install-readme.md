Summary
- Added explicit install instructions to the root README and `fixture3-ddmin` crate README.
- Bumped `fixture3-ddmin` to `0.1.1` so crates.io can receive the README update.

Decisions made
- Added both `cargo add fixture3-ddmin` and manual `[dependencies]` instructions because humans and agents use both paths.
- Documented the Rust import name `fixture3_ddmin` because the package name and crate name differ.
- Updated the root README as well as the crate README so both GitHub and crates.io entry points show the library install path.

Key files for context
- `README.md`
- `packages/ddmin/crates/ddmin/README.md`
- `packages/ddmin/crates/ddmin/Cargo.toml`
- `apps/fixtures/Cargo.lock`
- `packages/ddmin/Cargo.lock`
- `packages/ddmin/scripts/rust-probe/Cargo.lock`

Verification
- `scripts/verify-ddmin.sh`
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --manifest-path packages/ddmin/Cargo.toml -p fixture3-ddmin`
- `cargo publish --dry-run --allow-dirty --manifest-path packages/ddmin/crates/ddmin/Cargo.toml`
- `python3 scripts/verify-rust-workspaces.py --fmt`
- `python3 scripts/verify-rust-workspaces.py --check`
- `python3 scripts/verify-rust-workspaces.py --clippy`
- `python3 scripts/verify-rust-workspaces.py --g3rs`
- `scripts/verify-all.sh`

Next steps
- Push `development`, merge through PR CI into `main`, then publish `fixture3-ddmin 0.1.1` from `main`.
