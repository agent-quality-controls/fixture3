# Remove Output And Normalizer Manifest Surface

## Goal

Simplify fixture3's public model to:

```text
fixture3(fixtures, project_runner, approved_output, received_output)
```

The suite command is the project runner. It must write JSON to stdout. fixture3 always parses that JSON and writes canonical pretty JSON before comparison.

No manifest field should describe output format or external normalization because fixture3 has one foreseeable output contract: JSON stdout from the suite command.

## Current Problem

Current manifest has this public surface:

```yaml
output:
  format: "json"
  normalizer:
    argv:
      - "python3"
      - "scripts/normalize-output.py"
```

This is unnecessary and confusing because:

- `format: json` is the only supported format.
- `normalizer` creates a second command stage.
- Any project-specific cleanup can be moved into `suite.command.argv`.
- Agents confuse project runner, normalizer, and fixture3's internal JSON canonicalization.
- The V1 plan deferred external normalizers, but the broader architecture manifest pulled them back in.

## Target Public Manifest

Remove `output` entirely.

Target:

```yaml
version: 1
suites:
  my-suite:
    tags:
      - "example"
    fixtures:
      - "behavior/fixtures/my-suite/*/input.json"
    command:
      argv:
        - "python3"
        - "scripts/my-runner.py"
        - "{fixtures}"
      ok_exit_codes:
        - 0
    storage:
      approved_dir: "behavior/approved/my-suite"
      received_dir: ".fixture3/my-suite"
      diff_dir: ".fixture3/my-suite"
```

Public rule:

```text
suite.command.argv must write JSON to stdout
```

## Target Rust Types

In `apps/fixtures/crates/fixture3/src/manifest.rs`, change:

```rust
pub(crate) struct SuiteConfig {
    pub(crate) tags: Vec<String>,
    pub(crate) fixtures: Vec<String>,
    pub(crate) command: CommandConfig,
    pub(crate) output: OutputConfig,
    pub(crate) storage: StorageConfig,
}

pub(crate) struct OutputConfig {
    pub(crate) format: OutputFormat,
    pub(crate) normalizer: Option<NormalizerConfig>,
}

pub(crate) enum OutputFormat {
    Json,
}

pub(crate) struct NormalizerConfig {
    pub(crate) argv: Vec<String>,
}
```

To:

```rust
pub(crate) struct SuiteConfig {
    pub(crate) tags: Vec<String>,
    pub(crate) fixtures: Vec<String>,
    pub(crate) command: CommandConfig,
    pub(crate) storage: StorageConfig,
}
```

Remove:

```rust
OutputConfig
OutputFormat
NormalizerConfig
```

## Target Normalize API

Keep the module and function name `normalize`.

Change:

```rust
pub(crate) fn normalize(output: &[u8], config: &OutputConfig) -> Result<String, AppError>
```

To:

```rust
pub(crate) fn normalize(output: &[u8]) -> Result<String, AppError>
```

Behavior:

- Parse `output` as JSON.
- Pretty-print canonical JSON.
- Return canonical JSON with trailing newline.
- Error message must say the suite command stdout must be JSON.

Do not rename `normalize.rs` in this change.

## Code Changes

### `manifest.rs`

- Remove `SuiteConfig.output`.
- Remove `OutputConfig`.
- Remove `OutputFormat`.
- Remove `NormalizerConfig`.
- Make unknown `output` YAML fields fail by default through serde's existing struct behavior if possible. If unknown fields are currently accepted, add `#[serde(deny_unknown_fields)]` to manifest config structs.

### `normalize.rs`

- Remove external normalizer command execution.
- Remove dependency on `OutputConfig`.
- Keep `normalize(output: &[u8])`.

### `app.rs`

- Change check path from:

```rust
crate::normalize::normalize(&command_output.stdout, &suite.output)
```

To:

```rust
crate::normalize::normalize(&command_output.stdout)
```

### `command.rs`

- Remove `run_stdin_command` if no other caller remains.
- Remove stdin command plumbing if no other caller remains.
- Keep fixture command execution.

### `doctor.rs`

- Remove `normalizer_empty` check.
- Remove normalizer wording.
- Keep checks for command argv, fixtures, approved files, and storage collisions.

### `metadata.rs`

- Remove `normalizer_hash`.
- Remove comparison of `normalizer` hash.
- Keep:

```rust
schema_version
run_id
run_commit
fixture_hash
manifest_hash
tool_version
```

- No replacement hash is needed.

### `args.rs`

- Remove all public help references to:
  - `output`
  - `output.format`
  - `output.normalizer`
  - `normalizer`
- Add direct wording:
  - "The suite command must write JSON to stdout."
  - "fixture3 parses suite command stdout as JSON and writes canonical pretty JSON before comparison."

### `README.md`

- Remove `output:` from all manifest examples.
- Remove normalizer explanation.
- Add the fixed JSON stdout contract.

### `scaffold.rs` / init / new suite

- Remove generated `output:` blocks from scaffolded manifests.
- Generated project runner still emits JSON.

### `rewrite.rs`

- Trial manifest rewriting must no longer preserve or write `output`.

## Fixture And Verification Changes

Remove self normalizer-only fixture:

```text
behavior/fixtures/self/cases/normalizer/
scripts/self-case-normalize-json.py
```

Remove normalizer case from:

```text
scripts/self-check-harness.py
```

Update every fixture manifest to remove:

```yaml
output:
  format: "json"
```

or:

```yaml
output:
  format: "json"
  normalizer: ...
```

Known areas:

- `behavior/fixtures/self/cases/**/fixture3.yaml`
- `examples/fake-project/**/*.yaml`
- reducer fake-project manifests
- generated init/new-suite expected text
- `.plans/*.manifest.toml` rows that require output or normalizer paths/text
- CLI help manifest expectations

## Verifier Changes

Update manifest-driven verifiers so they require the new model:

- No manifest under `behavior/`, `examples/`, or root contains `output:`.
- No manifest contains `normalizer:`.
- CLI help contains "suite command must write JSON to stdout".
- CLI help does not contain `output.normalizer`.
- `scripts/verify-reducer.py` still passes.
- `scripts/verify-all.sh` passes.

## Migration Rule

No backward compatibility.

Any existing `fixture3.yaml` with `output:` should fail until migrated.

Project-specific cleanup must move into `command.argv`.

Example migration:

Before:

```yaml
command:
  argv:
    - "my-tool"
    - "{fixtures}"
output:
  format: "json"
  normalizer:
    argv:
      - "scripts/cleanup.py"
```

After:

```yaml
command:
  argv:
    - "scripts/run-and-cleanup.py"
    - "{fixtures}"
```

## Verification Commands

Run:

```bash
cargo fmt --manifest-path apps/fixtures/Cargo.toml --check
cargo check --manifest-path apps/fixtures/Cargo.toml
cargo clippy --manifest-path apps/fixtures/Cargo.toml --all-targets -- -D warnings
python3 scripts/verify-reducer.py
scripts/verify-all.sh
g3rs validate repo
```

Then reinstall locally:

```bash
cargo install --path apps/fixtures/crates/fixture3 --locked --force
```

Confirm:

```bash
fixture3 --help
fixture3 init --manifest .fixture3/probe.yaml
rg "output:|normalizer:" .fixture3/probe.yaml
```

The final `rg` must find nothing.
