# goldencheck

`goldencheck` is a CLI for fixture-based golden output checks.

It is independent of any one project. A project gives `goldencheck` a manifest that says which fixtures to run, which command to execute, how to normalize output, and where approved and received files live.

Use it when behavior is easiest to verify by comparing current command output against reviewed output stored in git. Typical uses are CLI output, parser output, generated JSON, diagnostics, migrations, rule engines, and API examples.

The full agent-oriented usage guide is built into the binary:

```bash
goldencheck --help
```

That help text explains the manifest schema, `{fixtures}` substitution, file layout, workflow, `--change`, and exit codes without requiring an agent to traverse subcommand help.

## Install

### GitHub Release

Download the prebuilt binary for your platform from the GitHub release:

```bash
curl -L -o goldencheck-aarch64-apple-darwin.tar.gz \
  https://github.com/websmasher/goldencheck/releases/download/v0.1.4/goldencheck-aarch64-apple-darwin.tar.gz
tar xzf goldencheck-aarch64-apple-darwin.tar.gz
install -m 0755 goldencheck ~/.cargo/bin/goldencheck
goldencheck --version
```

Available release targets:

- `aarch64-apple-darwin`
- `x86_64-apple-darwin`
- `aarch64-unknown-linux-gnu`
- `x86_64-unknown-linux-gnu`

### Cargo Binstall

Once the matching crates.io install stub is published, install with:

```bash
cargo install cargo-binstall
cargo binstall goldencheck
```

Do not use `cargo install goldencheck` for the real binary. The crates.io package is an install stub that exists so `cargo binstall goldencheck` can resolve release metadata and download prebuilt binaries.

## Manifest

Default manifest path:

```text
goldencheck.yaml
```

Example:

```yaml
version: 1
suites:
  lint-rules:
    fixtures:
      - "behavior/fixtures/lint-rules/*/input.json"
    command:
      argv:
        - "scripts/replay-fixture.sh"
        - "{fixtures}"
      ok_exit_codes:
        - 0
        - 1
    output:
      format: "json"
      normalizer:
        argv:
          - "scripts/normalize-output.py"
    storage:
      approved_dir: "behavior/golden/lint-rules"
      received_dir: ".goldencheck/lint-rules"
      diff_dir: ".goldencheck/lint-rules"
```

`{fixtures}` is replaced with the discovered fixture paths in deterministic order.

The normalizer is optional. When present, `goldencheck` writes the command stdout to the normalizer stdin and reads normalized output from normalizer stdout.

## Commands

Run one suite:

```bash
goldencheck check --suite lint-rules
goldencheck check --suite lint-rules --manifest goldencheck.yaml
```

Run every suite:

```bash
goldencheck check --all
goldencheck check --all --manifest goldencheck.yaml
```

Exit codes:

- `0`: received output matches approved output
- `1`: received output differs from approved output
- `2`: tool, config, command, or runtime error

For `check --all`, exit `2` wins over exit `1`. That means one errored suite returns `2` even if another suite only differs.

Show the latest diff without rerunning the suite:

```bash
goldencheck diff --suite lint-rules
```

Refresh and then show the diff:

```bash
goldencheck diff --suite lint-rules --refresh
```

Approve a received output:

```bash
goldencheck approve --suite lint-rules
```

If the received output differs from approved output, `--change` is required:

```bash
goldencheck approve --suite lint-rules --change behavior/changes/2026-05-13-change.md
```

Show suite state:

```bash
goldencheck status
goldencheck status --suite lint-rules
goldencheck status --all
```

Create an example manifest:

```bash
goldencheck init
```

## Files

Committed files:

```text
behavior/
  fixtures/
  golden/
  changes/
```

Generated files:

```text
.goldencheck/
  <suite>/
    received.raw.json
    received.normalized.json
    received.meta.json
    diff.json
    diff.txt
```

Approved output:

```text
behavior/golden/<suite>/
  approved.normalized.json
  approved.meta.json
```

## Fail-Closed Behavior

`goldencheck check` fails with exit `2` when:

- approved output is missing
- the project command exits with a code outside `ok_exit_codes`
- the normalizer exits non-zero
- command output or normalized output is invalid JSON
- approved metadata exists and fixture, manifest, or normalizer hashes changed

## Repository Verification

This repository does not use Rust tests.

Run:

```bash
scripts/verify-all.sh
```

The verifier checks:

- required tree shape
- no Rust test files or test attributes
- required G3RS config
- module dependency rules
- formatting, compilation, clippy, and G3RS
- self-hosted golden behavior
- CLI command contract
