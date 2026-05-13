# Summary

Expanded CLI help so users can understand what `goldencheck` is for and how to use it without already knowing the model.

The manifest verifier now checks required help text for the top-level command and each subcommand, so the CLI cannot regress to command-name-only help.

# Decisions Made

- Added long help to the top-level CLI explaining fixture commands, approved output, workflow, suites, and exit codes.
- Added long help to `check`, `diff`, `approve`, `status`, and `init`.
- Added flag help for `--suite`, `--manifest`, `--refresh`, and `--change`.
- Added `[[cli_help]]` and `help_contains` manifest checks instead of relying on manual review.
- Bumped `goldencheck` and `goldencheck-cli` to `0.1.1` because the published `0.1.0` help text was insufficient.

# Key Files For Context

- `crates/goldencheck/src/args.rs`
- `.plans/2026-05-13-150929-goldencheck-architecture.md.manifest.toml`
- `scripts/verify-manifest.py`
- `crates/goldencheck/Cargo.toml`
- `crates/goldencheck-install/Cargo.toml`

# Verification

```bash
scripts/verify-all.sh
cargo run -p goldencheck-cli -- --help
cargo run -p goldencheck-cli -- approve --help
```

Results:

```text
scripts/verify-all.sh: PASS
top-level help explains workflow, suites, manifest, storage, and exit codes
approve help explains received output, approved output, metadata, and --change
```

# Next Steps

- Tag and publish `v0.1.1`.
- Confirm `cargo binstall goldencheck` installs the 0.1.1 binary and `goldencheck --help` shows the expanded help.
