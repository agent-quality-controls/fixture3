Goal
- Make `goldencheck --help` enough for an agent or user to understand what the tool is for and how to use it without opening subcommand help first.

Approach
- Expand top-level help in `crates/goldencheck/src/args.rs`.
- Include the generated manifest example shape, `{fixtures}` substitution, supported output format, storage file layout, workflow, `--change`, and exit codes.
- Update the manifest help contract so the verifier checks the added content.
- Bump packages to `0.1.3` because `0.1.2` is already published.

Key decisions
- Keep the details in top-level help instead of forcing traversal into `init --help`.
- State that `json` is the only current output format because the manifest type only supports `OutputFormat::Json`.
- Describe `--change` as a reviewed change path stored in approved metadata, not as a file the tool interprets.

Files to modify
- `crates/goldencheck/src/args.rs`
- `.plans/2026-05-13-150929-goldencheck-architecture.md.manifest.toml`
- `crates/goldencheck/Cargo.toml`
- `crates/goldencheck-install/Cargo.toml`
- `Cargo.lock`
