# Goal

Document `fixture3` usage and add a release shape where users install the real binary through `cargo binstall`.

The crates.io package named `fixture3` should be a small install stub. The real CLI implementation should build as `fixture3-cli` and publish GitHub release archives for `cargo binstall fixture3`.

# Approach

1. Add a root `README.md` explaining install, manifest format, commands, generated files, approval flow, and verification.
2. Rename the implementation package from `fixture3` to `fixture3-cli` while keeping the binary name `fixture3`.
3. Add `crates/fixture3-install` as the crates.io package named `fixture3`; its binary prints a direct `cargo binstall fixture3` instruction.
4. Add `[package.metadata.binstall]` to the stub package so `cargo binstall fixture3` can resolve GitHub release artifacts.
5. Add release workflows based on the local Guardrail3 setup:
   - `release-plz` on `production`
   - binary release on `v*` tags
   - CI on `main` and pull requests
6. Update manifest-driven verifier rows for the new tree shape and command names.

# Key Decisions

- Keep the real binary name `fixture3`; only the implementation package name changes to avoid a crates.io name collision with the install stub.
- Keep the install stub dependency-free.
- Do not use `cargo test` anywhere.
- Keep package artifacts as tar.gz archives with the binary at archive root, matching the `bin-dir` in binstall metadata.

# Files To Modify

- `README.md`
- `Cargo.toml`
- `crates/fixture3/Cargo.toml`
- `crates/fixture3-install/Cargo.toml`
- `crates/fixture3-install/src/main.rs`
- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`
- `.github/workflows/binary-release.yml`
- `release-plz.toml`
- `fixture3.yaml`
- `.plans/2026-05-13-150929-fixture3-architecture.md.manifest.toml`
- self fixtures and verifier scripts as needed
