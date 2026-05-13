# Summary

Added user-facing README documentation and a release layout where `cargo binstall goldencheck` installs prebuilt GitHub release binaries.

The crates.io package named `goldencheck` is now a dependency-free install stub. The real CLI implementation package is `goldencheck-cli`, while the produced binary remains named `goldencheck`.

# Decisions Made

- Kept the runtime command name as `goldencheck` because that is the public CLI surface.
- Renamed the implementation package to `goldencheck-cli` so the crates.io package name `goldencheck` can be used for the install stub.
- Added `[package.metadata.binstall]` to the stub package with release artifact names shaped as `goldencheck-{target}.tar.gz`.
- Mirrored the local Guardrail3 release setup: release-plz on `production`, binary release on `v*` tags, and CI on `main` and pull requests.
- Removed `cargo test` from CI because this repository uses golden fixtures and static checks instead of Rust tests.

# Key Files For Context

- `README.md`
- `Cargo.toml`
- `crates/goldencheck/Cargo.toml`
- `crates/goldencheck-install/Cargo.toml`
- `crates/goldencheck-install/src/main.rs`
- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`
- `.github/workflows/binary-release.yml`
- `release-plz.toml`
- `.plans/2026-05-13-182528-readme-release-binstall.md`

# Verification

```bash
scripts/verify-all.sh
cargo publish -p goldencheck --dry-run --allow-dirty
cargo run -p goldencheck --bin goldencheck
cargo run -p goldencheck-cli --bin goldencheck -- --help
```

Results:

```text
scripts/verify-all.sh: PASS
cargo publish -p goldencheck --dry-run --allow-dirty: PASS
cargo run -p goldencheck --bin goldencheck: exits 1 and prints binstall instructions
cargo run -p goldencheck-cli --bin goldencheck -- --help: exits 0 and prints CLI help
```

# Next Steps

- Configure a Git remote before pushing.
- Confirm the exact GitHub repository URL before the first tag release so `cargo-binstall` artifact URLs resolve.
