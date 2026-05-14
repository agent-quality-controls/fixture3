Summary
- Expanded top-level `fixture3 --help` so it explains purpose, use cases, manifest schema, `{fixtures}` substitution, JSON output handling, storage files, workflow, `--change`, and exit codes.
- Added manifest verifier coverage for the new top-level help contract and bumped both packages to `0.1.3`.

Decisions made
- Kept the full usage model in top-level help because agents should not have to traverse subcommand help to understand the tool.
- Documented only `json` as supported output format because the manifest type only accepts `OutputFormat::Json`.
- Described `--change` as a reviewed path recorded in `approved.meta.json`; the tool does not read the path.

Key files for context
- `crates/fixture3/src/args.rs`
- `.plans/2026-05-13-150929-fixture3-architecture.md.manifest.toml`
- `.plans/2026-05-13-213039-frontload-cli-help.md`
- `crates/fixture3/Cargo.toml`
- `crates/fixture3-install/Cargo.toml`

Verification
- `cargo check`
- `cargo run -p fixture3-cli -- --help`
- `cargo run -p fixture3-cli -- --version`
- `scripts/verify-all.sh`
- `cargo publish -p fixture3 --dry-run --allow-dirty`

Next steps
- Commit and push the `0.1.3` help expansion.
- Tag `v0.1.3`.
- Wait for GitHub binary release assets.
- Publish `fixture3 v0.1.3` to crates.io.
- Reinstall through `cargo binstall fixture3` and verify the installed help.
