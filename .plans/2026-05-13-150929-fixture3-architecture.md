# Goal

Build `fixture3`: a reusable CLI for fixture-based approval testing.

The tool replaces repo-specific ad hoc replay scripts where the same pattern repeats:

```text
fixtures -> project command -> normalized received output -> approved output -> diff -> approve reviewed change
```

# Hard Verification Rule

This repository must not use Rust tests.

Forbidden:

- `#[test]`
- `#[cfg(test)]`
- `tests/`
- `cargo test`
- unit tests
- integration tests
- doc tests

Allowed:

- fixture behavior fixtures
- approved output files
- received output files
- deterministic diff output
- `cargo check`
- `cargo clippy`
- `cargo fmt --check`
- `g3rs validate --path . --rules-only`

The first runnable version of `fixture3` must check its own fixture suite. Before the CLI can self-host, bootstrap verification may use shell scripts that inspect the repository and run static tools. Those scripts are temporary scaffolding, not tests.

# Name

Use `fixture3`.

Reason:

- `approved` maps to the established "approved file" / snapshot testing approach.
- `check` says the tool verifies current output against recorded output.
- It is readable as a command: `fixture3 check`, `fixture3 diff`, `fixture3 approve`.

# Core Model

The tool does not know what a fixture means.

The project gives it:

- fixture globs
- command to run
- acceptable exit codes
- optional normalizer command
- where to store approved output and received output
- output identity keys for semantic diffing

The tool owns:

- deterministic fixture discovery
- command execution
- run metadata
- fixture hash
- manifest hash
- normalizer hash
- approved output storage
- received output storage
- normalized comparison
- diff reporting
- approve gating

# Non-Goals

- Do not implement project-specific assertions.
- Do not discover fixtures without a manifest.
- Do not require Rust projects.
- Do not depend on `insta`.
- Do not parse every possible output format in V1.
- Do not build a UI.
- Do not replace test runners.
- Do not add Rust tests to this repository.

# Manifest

Default path:

```text
fixture3.yaml
```

Example:

```yaml
version: 1

suites:
  textlint-rules:
    fixtures:
      - "behavior/fixtures/textlint-rules/*/family.md"

    command:
      argv:
        - "scripts/behavior-replay.sh"
        - "{fixtures}"
      ok_exit_codes: [0, 1]

    output:
      format: "json"
      normalizer:
        argv:
          - "scripts/normalize-textlint-output.js"

    storage:
      approved_dir: "behavior/approved/textlint-rules"
      received_dir: ".fixture3/textlint-rules"
      diff_dir: ".fixture3/textlint-rules"

    identity:
      fixture_id: "path"
      record_keys:
        - "filePath"
        - "messages[].ruleId"
        - "messages[].message"
        - "messages[].line"
        - "messages[].column"
```

V1 can treat `record_keys` as advisory and start with normalized JSON structural diff. Semantic keyed diff can be V1.1 if needed.

# Commands

## `fixture3 check`

Runs one suite, stores received output, compares it to approved output, and exits according to the comparison result.

```bash
fixture3 check --suite textlint-rules
fixture3 check --suite textlint-rules --manifest fixture3.yaml
```

Output:

```text
suite: textlint-rules
received_run_id: 2026-05-13T15-09-29Z-8e4a1c
received_run_commit: 910cf29
fixtures: 9
status: matched
```

Stores:

```text
.fixture3/textlint-rules/
  received.raw.json
  received.normalized.json
  received.meta.json
  diff.json
  diff.txt
```

Exit codes:

- `0`: approved and received match
- `1`: approved and received differ
- `2`: tool/config/runtime error

## `fixture3 diff`

Shows the last diff from `check`.

```bash
fixture3 diff --suite textlint-rules
fixture3 diff --suite textlint-rules --refresh
```

Rules:

- Without `--refresh`, it does not run the project command.
- With `--refresh`, it runs `check` first and then prints the diff.

Output:

```text
suite: textlint-rules
approved_run_commit: 910cf29
received_run_commit: dirty
fixtures: 9
added: 2
removed: 0
changed: 1
```

Stores:

```text
.fixture3/textlint-rules/diff.json
.fixture3/textlint-rules/diff.txt
```

## `fixture3 approve`

Publishes received output as the new approved output.

```bash
fixture3 approve --suite textlint-rules --change behavior/changes/2026-05-13-cliches.yaml
```

Rules:

- If there is no diff, `--change` is optional.
- If there is a diff, `--change` is required.
- V1 records the change path in metadata.
- V1 does not need full change classification yet.

Stores:

```text
behavior/approved/textlint-rules/
  approved.normalized.json
  approved.meta.json
```

## `fixture3 status`

Reports suite state.

```bash
fixture3 status
fixture3 status --suite textlint-rules
```

Shows:

- approved exists or missing
- approved run commit
- approved recorded time
- fixture hash status
- manifest hash status
- normalizer hash status
- latest received run
- diff count
- pending change file if attached

## `fixture3 init`

Writes a small example manifest and directory skeleton.

```bash
fixture3 init
```

# Metadata

Every generated output has metadata:

```json
{
  "suite": "textlint-rules",
  "kind": "received",
  "run_id": "2026-05-13T15-09-29Z-8e4a1c",
  "run_commit": "910cf29",
  "working_tree": "dirty",
  "recorded_at": "2026-05-13T15:09:29Z",
  "fixture_hash": "sha256:...",
  "manifest_hash": "sha256:...",
  "normalizer_hash": "sha256:...",
  "tool_version": "0.1.0",
  "output_schema_version": "1"
}
```

Use `run_commit`, not `accepted_commit` or `candidate_commit`.

Reason: the tool knows which commit generated an output. The role is expressed by `kind`: `approved` or `received`.

# Fail-Closed Rules

Diff must fail with a tool error when:

- approved output is missing
- fixture hash changed and no fixture-change mode is provided
- manifest hash changed and no manifest-change mode is provided
- normalizer hash changed and no normalizer-change mode is provided
- command exits with a code outside `ok_exit_codes`
- normalizer exits non-zero

V1 can expose explicit override flags:

```bash
fixture3 diff --suite textlint-rules --allow-fixture-change
fixture3 diff --suite textlint-rules --allow-manifest-change
fixture3 diff --suite textlint-rules --allow-normalizer-change
```

# Storage

Approved files are stored in git.

Suggested layout:

```text
behavior/
  fixtures/
  approved/
  changes/
  schemas/
.fixture3/
  runs/
  diffs/
```

# Architecture

Implement as a Rust CLI with small internal modules:

```text
fixture3/
  Cargo.toml
  crates/fixture3/
    Cargo.toml
    src/
      main.rs
      args.rs
      manifest.rs
      fixture.rs
      command.rs
      normalize.rs
      metadata.rs
      storage.rs
      diff.rs
      approve.rs
      git.rs
      error.rs
```

Dependencies:

- `clap` for CLI args
- `serde`, `serde_json`, `serde_yaml` for config/output
- `globset` or `globwalk` for fixture discovery
- `sha2` for hashes
- `time` for timestamps
- `similar` for readable text diffs if needed
- `thiserror` for errors

Avoid:

- plugins
- async runtime
- embedded scripting language
- database
- service mode
- Rust tests

# G3RS

Use the current G3RS config file:

```text
guardrail3-rs.toml
```

Required policy:

```toml
[checks]
test = false
```

The G3RS test family is disabled because the project deliberately bans test files and test functions. Static G3RS validation still runs through:

```bash
g3rs validate --path . --rules-only
```

# First Integration Target

Use `prosesmasher` textlint fixtures as the first real external integration:

```yaml
suites:
  textlint-rules:
    fixtures:
      - "behavior/fixtures/textlint-rules/*/family.md"
    command:
      argv: ["scripts/behavior-replay.sh", "{fixtures}"]
      ok_exit_codes: [0, 1]
    output:
      format: "json"
    storage:
      approved_dir: "behavior/approved/textlint-rules"
      received_dir: ".fixture3/textlint-rules"
      diff_dir: ".fixture3/textlint-rules"
```

Current scripts can remain during migration. `fixture3` should first reproduce `scripts/behavior-verify.sh` behavior, then add metadata and approve gating.

# Self-Hosting Target

After the first runnable command path exists, add a `fixture3-self` suite that runs the built CLI against local sample fixtures.

The self suite must be the project's verification path. Do not add Rust tests as a parallel proof system.

# V1 Definition Of Done

- `fixture3 check --suite textlint-rules` writes received output and returns `0` when received matches approved.
- `fixture3 check --suite textlint-rules` returns `1` and writes a diff when received differs from approved.
- `fixture3 diff --suite textlint-rules` prints the latest diff without rerunning unless `--refresh` is passed.
- `fixture3 approve --suite textlint-rules` publishes a no-diff received output as approved.
- `fixture3 approve --suite textlint-rules --change <path>` publishes a changed received output as approved.
- Metadata includes `run_commit`, `working_tree`, fixture hash, manifest hash, normalizer hash, and tool version.
- Existing ad hoc scripts can be replaced or wrapped by fixture3 after parity is proven.
- No Rust tests exist in the repository.
