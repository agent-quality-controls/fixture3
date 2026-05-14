# fixture3

`fixture3` is a CLI for fixture-based approval testing in agent-managed codebases.

Install:

```bash
cargo install cargo-binstall
cargo binstall fixture3
fixture3 --version
```

Use `cargo binstall fixture3` as the install path. The crates.io package is an install stub for cargo-binstall metadata, not the real CLI implementation.

The full agent guide is in `fixture3 --help`. It explains the model, manifest schema, fixture substitution, files, feature selectors, JSON output, approval flow, and exit codes from the top-level help screen.

## Why this exists

Unit tests can be a bad fit for large agent-managed codebases. For behavior-heavy code, the test code can grow until it is as large as the production code. Then an agent has two equally easy ways to make a broken change pass: change the app back to the intended behavior, or rewrite the tests to accept the broken behavior.

`fixture3` moves the trust boundary. Fixtures are stable inputs that describe the behavior surface layer by layer. Approved outputs are the reviewed behavior for the current accepted commit. When code changes, the inputs usually stay put and only the received output changes.

That makes review smaller. Instead of judging a rewritten test suite, a reviewer can inspect the behavior diff: previous approved output against new received output. Agents are much better at reviewing a concrete output diff than guessing intent from changed test code.

## The model

`fixture3` manages approval plumbing. The project owns meaning.

- A fixture is an input file.
- A suite is one approval check over fixture inputs.
- A feature is a named group of suites, usually tied to a spec path.
- Tags are loose suite labels for operational groups.
- Approved output is committed reviewed behavior.
- Received output is the latest command output.
- Diff output is the review surface.

Features and tags do not teach `fixture3` what your app does. They give agents and humans stable handles for running the right behavior slice.

## Manifest

A project defines suites, tags, and features in `fixture3.yaml`:

```yaml
version: 1
features:
  linting:
    spec: "docs/features/linting.md"
    suites:
      - "lint-rules"
suites:
  lint-rules:
    tags:
      - "lint"
      - "rules"
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
      approved_dir: "behavior/approved/lint-rules"
      received_dir: ".fixture3/lint-rules"
      diff_dir: ".fixture3/lint-rules"
```

`{fixtures}` is replaced with discovered fixture paths in deterministic order. If an arg is exactly `{fixtures}`, each fixture becomes a separate argv item. If it appears inside a larger arg, fixture paths are joined with spaces.

The only supported output format is JSON. A normalizer is optional. When present, `fixture3` writes command stdout to the normalizer stdin and reads normalized JSON from normalizer stdout.

## Files

Committed files:

```text
behavior/
  fixtures/
  approved/
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
behavior/approved/<suite>/
  approved.normalized.json
  approved.meta.json
```

## Workflow

Create a starter manifest:

```bash
fixture3 init
```

Create suite scaffolding:

```bash
fixture3 new suite lint-rules
```

`new suite` creates a sample fixture and an initial approved output, then prints the manifest block to add under `suites:`. It does not edit `fixture3.yaml`; the project keeps ownership of feature grouping and manifest formatting.

Inspect setup before running behavior:

```bash
fixture3 explain --suite lint-rules
fixture3 doctor
```

Run behavior:

```bash
fixture3 check --suite lint-rules
fixture3 check --all
fixture3 check --tag lint
fixture3 check --feature linting
```

Review drift:

```bash
fixture3 diff --suite lint-rules
fixture3 diff --suite lint-rules --refresh
```

Approve a reviewed change:

```bash
fixture3 approve --suite lint-rules --change behavior/changes/2026-05-14-rule-change.md
```

Show state:

```bash
fixture3 status
fixture3 status --suite lint-rules
fixture3 status --tag lint
fixture3 status --feature linting
fixture3 status --all
```

## Agent output

Use JSON when another tool or agent needs to consume state without parsing terminal text:

```bash
fixture3 check --feature linting --json
fixture3 status --all --json
fixture3 diff --suite lint-rules --json
fixture3 explain --suite lint-rules --json
fixture3 doctor --json
```

`check --json` writes one record per selected suite with status, exit code, fixture count, received path, diff path, and error text. `status --json` writes approved, received, and diff booleans. `diff --json` writes diff status and text. `doctor --json` writes setup findings.

## Commands

- `fixture3 init`: writes an example `fixture3.yaml`.
- `fixture3 new suite <name>`: creates fixture and approved-output scaffolding and prints a manifest block.
- `fixture3 check --suite <suite>`: runs one suite.
- `fixture3 check --all`: runs every suite.
- `fixture3 check --tag <tag>`: runs suites with a tag.
- `fixture3 check --feature <feature>`: runs suites listed under a feature.
- `fixture3 diff --suite <suite>`: shows the latest stored diff.
- `fixture3 diff --suite <suite> --refresh`: reruns the suite before showing the diff.
- `fixture3 approve --suite <suite> --change <path>`: promotes received output to approved output.
- `fixture3 status`: lists state for every suite.
- `fixture3 explain --suite <suite>`: shows resolved fixture globs, fixture count, command argv, tags, feature membership, storage paths, and file state.
- `fixture3 doctor`: validates manifest shape without running project behavior.

Exit codes:

- `0`: received output matches approved output, or setup validation passed.
- `1`: received output differs from approved output.
- `2`: tool, config, command, normalizer, manifest, or runtime error.

For multi-suite checks, exit `2` wins over exit `1`.

## Fail-closed checks

`fixture3 check` exits `2` when:

- approved output is missing
- the project command exits with a code outside `ok_exit_codes`
- the normalizer exits non-zero
- command output or normalized output is invalid JSON
- approved metadata exists and fixture, manifest, or normalizer hashes changed

`fixture3 doctor` exits `2` when:

- a feature references a missing suite
- fixture globs are invalid or match no files
- command argv or exit-code lists are empty
- normalizer argv is empty
- approved output is missing
- storage paths collide

## Repository verification

This repository uses its own fixture suite instead of Rust tests.

```bash
scripts/verify-all.sh
```

The verifier checks the tree, forbidden test files, config, module dependencies, formatting, compilation, clippy, G3RS, self-hosted fixture behavior, CLI help, and the feature-pipeline contract.
