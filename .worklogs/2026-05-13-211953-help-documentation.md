# Summary

Expanded CLI help so users can understand what `fixture3` is for and how to use it without already knowing the model.

The manifest verifier now checks required help text for the top-level command and each subcommand, so the CLI cannot regress to command-name-only help.

# Decisions Made

- Added long help to the top-level CLI explaining fixture commands, approved output, workflow, suites, and exit codes.
- Added long help to `check`, `diff`, `approve`, `status`, and `init`.
- Added flag help for `--suite`, `--manifest`, `--refresh`, and `--change`.
- Added `[[cli_help]]` and `help_contains` manifest checks instead of relying on manual review.
- Bumped `fixture3` and `fixture3-cli` to `0.1.1` because the published `0.1.0` help text was insufficient.

# Key Files For Context

- `crates/fixture3/src/args.rs`
- `.plans/2026-05-13-150929-fixture3-architecture.md.manifest.toml`
- `scripts/verify-manifest.py`
- `crates/fixture3/Cargo.toml`
- `crates/fixture3-install/Cargo.toml`

# Verification

```bash
scripts/verify-all.sh
cargo run -p fixture3-cli -- --help
cargo run -p fixture3-cli -- approve --help
```

Results:

```text
scripts/verify-all.sh: PASS
top-level help explains workflow, suites, manifest, storage, and exit codes
approve help explains received output, approved output, metadata, and --change
```

# Next Steps

- Tag and publish `v0.1.1`.
- Confirm `cargo binstall fixture3` installs the 0.1.1 binary and `fixture3 --help` shows the expanded help.
