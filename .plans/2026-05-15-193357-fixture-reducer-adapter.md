# Fixture Reducer Adapter Plan

## Goal

Add a `fixture3 reduce` command that uses `fixture3-ddmin` to remove unnecessary files from a copied fixture project while preserving an explicit fixture output contract.

The first reducer pass is file-tree reduction only. It does not reduce TOML fields, source-code spans, dependency edges, or directory names.

The command is useful when a fixture was copied from a real project and contains files that do not affect the behavior suite being checked.

## Product Contract

Command:

```bash
fixture3 reduce --suite <suite> --manifest <path> --fixture-root <path> --work-dir <path>
```

Required arguments:

- `--suite <suite>` names the suite whose behavior must be preserved.
- `--manifest <path>` defaults to `fixture3.yaml` and is resolved relative to the current working directory.
- `--fixture-root <path>` is the directory tree to reduce.
- `--work-dir <path>` is a scratch directory owned by fixture3 for trial trees and reports.

Output:

- Reduce writes JSON to stdout. There is no text mode.
- The same JSON object is written to `<work-dir>/reduce-report.json`.

Exit codes:

- `0` means the reducer completed and produced a report.
- `2` means manifest, fixture-root, work-dir, trial tree creation, or oracle execution failed.

The command never edits `fixture-root` directly.

## Behavior Contract

The oracle preserves the current approved output for the selected suite.

For each DDMin candidate set:

1. Create a trial tree under `work-dir/trials/<call-number>/fixture-root`.
2. Copy all files from `fixture-root` except removed candidate files.
3. Write a trial manifest under `work-dir/trials/<call-number>/manifest.fixture3.yaml`.
4. Rewrite only the selected suite fixture list so it contains exact remaining file paths inside the trial tree.
5. Rewrite only the selected suite received and diff storage to `work-dir/received/<suite>` and `work-dir/diff/<suite>`.
6. Keep the selected suite approved storage unchanged.
7. Run the same internal check path used by `fixture3 check --suite <suite>`.
8. Return `Interesting` when the check status is matched.
9. Return `NotInteresting` when the check status is different.
10. Return `Unresolved` when trial tree creation fails, fixture discovery fails, command execution errors, JSON normalization fails, approved files are missing, or the check returns a tool error.

This means reduction preserves the current approved output, not arbitrary future intent.

## Candidate Model

Candidate type:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FileCandidate {
    pub(crate) id: CandidateId,
    pub(crate) relative_path: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct CandidateId(u32);
```

Candidate collection rules:

- Walk `fixture-root` recursively.
- Include regular files only.
- Sort by relative path.
- Assign stable `CandidateId` after sorting.
- Exclude generated and VCS paths before candidate creation.

Hard excluded path components:

- `.git`
- `target`
- `.cargo-target`
- `node_modules`
- `dist`
- `.fixture3`
- `.goldencheck`

Symlink policy:

- Symlinked files and symlinked directories are `Unresolved` for this first version.
- The reducer reports the symlink path and stops before DDMin.
- This avoids platform-dependent copy behavior and accidental traversal outside the fixture root.

Empty directory policy:

- Empty directories are not candidates.
- Trial tree creation creates parent directories only for remaining files.
- Directory minimization is a later pass, not part of this plan.

## Trial Tree Policy

Module: `reduce/trial_tree.rs`

Trial tree creation uses a clean directory per oracle call:

```text
<work-dir>/trials/<call-number>/fixture-root/
<work-dir>/trials/<call-number>/manifest.fixture3.yaml
<work-dir>/trials/<call-number>/received/<suite>/
<work-dir>/trials/<call-number>/diff/<suite>/
```

Rules:

- Remove the call directory before writing it.
- Copy remaining regular files with `std::fs::copy` through the existing filesystem adapter or a reducer-specific filesystem adapter.
- Preserve relative paths.
- Do not preserve permissions beyond normal file copy defaults.
- Do not copy files outside `fixture-root`.
- Reject paths that escape the trial tree root after normalization.

## Manifest Rewrite Policy

Module: `reduce/rewrite.rs`

Rewrite from typed manifest data, not string replacement.

Rules:

- Load the original manifest with existing `manifest::load`.
- Select exactly one suite.
- Replace selected suite `fixtures` with exact file paths for each remaining candidate:

```text
<call-dir>/fixture-root/<relative-file>
```

- Replace selected suite `storage.received` with:

```text
<call-dir>/received/<suite>
```

- Replace selected suite `storage.diff` with:

```text
<call-dir>/diff/<suite>
```

- Keep selected suite `storage.approved` unchanged.
- Keep command, output, tags, and features unchanged.
- Serialize with `serde_norway`.

This does not preserve YAML comments or formatting in the temporary manifest. That is acceptable because the file is generated under `work-dir` only.

## Proof Report

Write proof files under:

```text
<work-dir>/reduce-report.json
<work-dir>/removed-files.txt
<work-dir>/remaining-files.txt
```

`reduce-report.json` schema:

```json
{
  "suite": "name",
  "fixture_root": "path",
  "work_dir": "path",
  "candidate_count": 12,
  "remaining_count": 4,
  "removed_count": 8,
  "oracle_calls": 31,
  "interesting_trials": 9,
  "not_interesting_trials": 20,
  "unresolved_trials": 2,
  "guarantee": "complete",
  "remaining_files": ["relative/path"],
  "removed_files": ["relative/path"]
}
```

Guarantee strings:

- `complete`
- `incomplete:max-oracle-calls-reached`
- `incomplete:baseline-not-interesting`

## File Structure

Add these CLI modules under the existing CLI crate:

```text
apps/fixtures/crates/fixture3/src/reduce/candidate.rs
apps/fixtures/crates/fixture3/src/reduce/mod.rs
apps/fixtures/crates/fixture3/src/reduce/run.rs
apps/fixtures/crates/fixture3/src/reduce/trial_tree.rs
apps/fixtures/crates/fixture3/src/reduce/oracle.rs
apps/fixtures/crates/fixture3/src/reduce/report.rs
apps/fixtures/crates/fixture3/src/reduce/rewrite.rs
```

Module ownership:

- `reduce/mod.rs`: facade-only module declarations and command re-export.
- `reduce/run.rs`: command implementation.
- `candidate.rs`: file walk, exclusions, candidate IDs.
- `trial_tree.rs`: per-call filesystem copy.
- `rewrite.rs`: temporary manifest generation.
- `oracle.rs`: adapter from trial tree to `OracleOutcome`.
- `report.rs`: reducer report data and writers.

Update existing files:

- `apps/fixtures/crates/fixture3/Cargo.toml`: add path dependency on `fixture3-ddmin`.
- `apps/fixtures/crates/fixture3/src/lib.rs`: export `reduce` internally.
- `apps/fixtures/crates/fixture3/src/args.rs`: add `reduce` command and help.
- `apps/fixtures/crates/fixture3/src/app.rs`: dispatch `reduce`.
- `apps/fixtures/guardrail3-rs.toml`: allow `fixture3-ddmin` if G3RS requires it.
- `scripts/verify-feature-pipeline.py`: include reduce help checks if CLI help verification is still split there.
- `.plans/2026-05-13-150929-fixture3-architecture.md.manifest.toml`: add reduce module and command expectations.

## Dependency Rule

The CLI may depend on `fixture3-ddmin` because `fixture3-ddmin` is marked shared in its own package metadata.

No other new runtime dependency is added.

Walk implementation uses the standard library in this pass because the current CLI already has a filesystem adapter and the exclusion policy is explicit and small. Do not add `ignore` or `walkdir` in this first adapter. The Guardrail3 crawler experience is relevant for project-wide source discovery, but this reducer is intentionally scoped to one user-provided fixture root with hard exclusions.

## Behavior Fixtures

Add fake project reducer cases under:

```text
examples/fake-project/behavior/fixtures/reducer-basic/project/
examples/fake-project/behavior/fixtures/reducer-basic/fixture3.yaml
examples/fake-project/behavior/approved/reducer-basic/approved.normalized.json
```

The fake project contains:

```text
project/keep/input.json
project/remove/noise.txt
project/remove/deep/noise.json
```

The verifier creates these generated or VCS paths in the copied run tree:

```text
project/.git/ignored.txt
project/target/ignored.txt
```

The suite command reads only `keep/input.json` and emits stable JSON.

Expected reducer result:

- `keep/input.json` remains.
- `remove/noise.txt` is removed.
- `remove/deep/noise.json` is removed.
- `.git/ignored.txt` and `target/ignored.txt` are excluded before candidate creation.
- `reduce-report.json` reports `candidate_count = 3`, `remaining_count = 1`, `removed_count = 2`, `guarantee = complete`.

Add a second case:

```text
examples/fake-project/behavior/fixtures/reducer-symlink/project/link
```

Expected result:

- `fixture3 reduce` exits `2`.
- stderr contains `symlink fixture content is not supported`.

## Verifier Updates

Add script:

```text
scripts/verify-reducer.py
```

The verifier:

1. Builds the CLI with `cargo build --manifest-path apps/fixtures/Cargo.toml -p fixture3-cli`.
2. Copies reducer fake projects into `.fixture3/reducer-run`.
3. Runs `fixture3 reduce` for reducer-basic.
4. Asserts report JSON counts and file lists.
5. Asserts original fixture root was not modified.
6. Runs `fixture3 reduce` for reducer-symlink.
7. Asserts exit code `2` and expected stderr.
8. Prints `reducer: PASS`.

Wire into:

```text
scripts/verify-all.sh
```

## CLI Help Requirements

Top-level help must include:

- `fixture3 reduce` minimizes copied fixture trees.
- Reduction preserves the selected suite's approved output.
- The command never edits `--fixture-root` directly.

Command help must include:

- `--suite`
- `--manifest`
- `--fixture-root`
- `--work-dir`
- Generated paths: `reduce-report.json`, `removed-files.txt`, `remaining-files.txt`.
- Exit code meaning for `0` and `2`.

## Implementation Order

1. Add manifest rows and verifier expectations for the reducer command.
2. Add CLI args and help text without implementation.
3. Add `reduce` module skeleton returning a clear not-implemented error.
4. Add candidate collection and symlink rejection.
5. Add trial tree creation.
6. Add manifest rewrite.
7. Add oracle adapter using existing internal check logic.
8. Add DDMin invocation and report writing.
9. Add reducer fake-project verifier.
10. Run full verification and fix only failures in scope.

## Done Gates

All commands must pass:

```bash
g3rs validate repo
python3 scripts/verify-rust-workspaces.py --g3rs
python3 scripts/verify-rust-workspaces.py --fmt
python3 scripts/verify-rust-workspaces.py --check
python3 scripts/verify-rust-workspaces.py --clippy
scripts/verify-ddmin.sh
scripts/verify-reducer.py
scripts/verify-all.sh
git diff --check
```
