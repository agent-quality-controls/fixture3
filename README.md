# fixture3

`fixture3` is a CLI for fixture-based approval testing in agent-managed codebases.

Install the current GitHub release:

```bash
curl -L -o fixture3-aarch64-apple-darwin.tar.gz \
  https://github.com/websmasher/fixture3/releases/download/v0.1.5/fixture3-aarch64-apple-darwin.tar.gz
tar xzf fixture3-aarch64-apple-darwin.tar.gz
install -m 0755 fixture3 ~/.cargo/bin/fixture3
fixture3 --version
```

Available release targets:

- `aarch64-apple-darwin`
- `x86_64-apple-darwin`
- `aarch64-unknown-linux-gnu`
- `x86_64-unknown-linux-gnu`

The full agent guide is in `fixture3 --help`. It explains the manifest schema, `{fixtures}` substitution, file layout, workflow, `--change`, and exit codes from the top-level help screen.

## Cargo binstall

After the matching crates.io install stub is published:

```bash
cargo install cargo-binstall
cargo binstall fixture3
```

`cargo install fixture3` installs only the stub package. The real binary is downloaded by `cargo binstall fixture3` from the GitHub release.

## Why this exists

Unit tests can be a bad fit for large agent-managed codebases. For behavior-heavy code, the test code can grow until it is as large as the production code. Then an agent has two equally easy ways to make a broken change pass: change the app back to the intended behavior, or rewrite the tests to accept the broken behavior.

`fixture3` moves the trust boundary. Fixtures are stable inputs that describe the behavior surface layer by layer. Approved outputs are the reviewed behavior for the current accepted commit. When code changes, the inputs usually stay put and only the received output changes.

That makes review smaller. Instead of judging a rewritten test suite, a reviewer can inspect the behavior diff: previous approved output against new received output. Agents are much better at reviewing a concrete output diff than guessing intent from changed test code.

## How it works

A project defines suites in `fixture3.yaml`. Each suite says:

- which fixture files are inputs
- which command runs against those fixtures
- which exit codes are accepted
- how stdout is normalized
- where approved, received, and diff files live

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
      received_dir: ".fixture3/lint-rules"
      diff_dir: ".fixture3/lint-rules"
```

`{fixtures}` is replaced with the discovered fixture paths in deterministic order.

The only supported output format is JSON. A normalizer is optional. When present, `fixture3` writes command stdout to the normalizer stdin and reads normalized JSON from normalizer stdout.

## Daily workflow

Create a manifest:

```bash
fixture3 init
```

Run one suite:

```bash
fixture3 check --suite lint-rules
fixture3 check --suite lint-rules --manifest fixture3.yaml
```

Run every suite:

```bash
fixture3 check --all
```

Show a stored diff:

```bash
fixture3 diff --suite lint-rules
```

Refresh and show a diff:

```bash
fixture3 diff --suite lint-rules --refresh
```

Approve a reviewed behavior change:

```bash
fixture3 approve --suite lint-rules --change behavior/changes/2026-05-14-rule-change.md
```

Show suite state:

```bash
fixture3 status
fixture3 status --suite lint-rules
fixture3 status --all
```

Exit codes:

- `0`: received output matches approved output
- `1`: received output differs from approved output
- `2`: tool, config, command, normalizer, or runtime error

For `check --all`, exit `2` wins over exit `1`.

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
.fixture3/
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

## Fail-closed checks

`fixture3 check` exits `2` when:

- approved output is missing
- the project command exits with a code outside `ok_exit_codes`
- the normalizer exits non-zero
- command output or normalized output is invalid JSON
- approved metadata exists and fixture, manifest, or normalizer hashes changed

## Repository verification

This repository uses its own fixture suite instead of Rust tests.

```bash
scripts/verify-all.sh
```

The verifier checks the tree, forbidden test files, config, module dependencies, formatting, compilation, clippy, G3RS, self-hosted fixture behavior, and CLI help.
