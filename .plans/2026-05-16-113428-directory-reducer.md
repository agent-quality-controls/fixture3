# Directory Reducer Plan

## Goal

Add a directory-level reducer pass before the existing file-level DDMin pass.

The purpose is to remove irrelevant fixture subtrees quickly. The Guardrail3 L45 probe showed that flat file DDMin removed only 37 of 303 files in 200 calls because path-order file chunks usually break the behavior contract. Directory candidates should remove whole irrelevant rule families with far fewer oracle calls.

This plan keeps the existing behavior contract:

- preserve the selected suite's approved output
- never edit `--fixture-root`
- write best-so-far files
- use one reusable `trial-current` directory
- obey `--max-oracle-calls`

## Command Contract

Add:

```bash
fixture3 reduce
fixture3 reduce --reducers dirs,files
fixture3 reduce --reducers dirs
fixture3 reduce --reducers files
```

Defaults:

- Default reducers are `dirs,files`.
- `dirs` runs directory-subtree reduction.
- `files` runs current file-level DDMin reduction.
- Reducers run left to right.
- One shared oracle-call budget applies across the whole reducer list.
- Unknown reducer names are a CLI error.
- Duplicate reducer names are rejected in this version.

Reason:

- A reducer list scales to future reducers such as `generated-dirs`, `toml`, `deps`, or `rust-code`.
- A special `--then-files` flag would encode one sequence as a one-off CLI shape.
- `dirs,files` should be default because the real-world copied-fixture case needs large subtree removal before file-level cleanup.

## Candidate Model

Add a generic reducer candidate enum:

```rust
pub(crate) enum ReduceCandidate {
    File(FileCandidate),
    Directory(DirectoryCandidate),
}
```

Directory candidate:

```rust
pub(crate) struct DirectoryCandidate {
    id: CandidateId,
    relative_path: PathBuf,
    file_count: usize,
}
```

Rules:

- Collect directories under `fixture-root`.
- Exclude the fixture root itself.
- Exclude directories with any excluded path component:
  - `.git`
  - `target`
  - `.cargo-target`
  - `node_modules`
  - `dist`
  - `.fixture3`
  - `.goldencheck`
- Reject symlinked directories before DDMin, same as files.
- Include only directories that contain at least one regular file after exclusions.
- Sort by ascending depth, then path.
- Assign stable `CandidateId` after sorting.

Why top-down:

- Parent and child directory candidates overlap.
- The primary goal is to remove large irrelevant subtrees quickly.
- If a parent directory is removable, testing children first wastes calls and reports less useful reductions.
- Top-down removal prunes all descendants from later candidate sets.
- Mixed required/removable parents will fail quickly, and the reducer then descends into their children.

## Overlap Policy

Directory candidates overlap by containment. The pass must make overlap explicit.

For a trial candidate set, a file is copied only when:

- it is not under any removed directory candidate
- and it remains after any previous accepted reductions

The implementation must not delete a child directory candidate separately after its parent is already removed.

Concrete representation:

- Keep a `BTreeSet<PathBuf>` of accepted removed directory paths.
- Before each oracle call, convert the DDMin `remaining directories` list into `removed directories`.
- A file is excluded if any ancestor path is in the removed directory set.

## Multi-Reducer State

Add reducer state type:

```rust
pub(crate) struct ReductionState {
    original_files: Vec<FileCandidate>,
    removed_directories: BTreeSet<PathBuf>,
    removed_files: BTreeSet<CandidateId>,
}
```

Rules:

- `dirs` mutates only `removed_directories`.
- `files` mutates only `removed_files`.
- Trial tree creation receives `ReductionState`.
- Final report is generated from the state, not only from the last DDMin output.

Reason:

- Each reducer must start from the state produced by previous reducers.
- The report must explain both directory removals and file removals.
- This avoids mixing two candidate universes into one flat list.

## Oracle Budget

`--max-oracle-calls` is shared across reducers.

Rules:

- `ReduceOracle` owns one call counter.
- Every reducer uses the same oracle instance or a shared budget object.
- If the budget is exhausted during one reducer, later reducers are skipped.
- Final guarantee is:
  - `complete` only if every requested reducer completed.
  - `incomplete:max-oracle-calls-reached` if the shared budget is exhausted.
  - `incomplete:baseline-not-interesting` if the initial baseline does not match.

Important gotcha:

- The DDMin crate currently accepts `max_oracle_calls` per run.
- For multiple reducers, passing the same limit to every DDMin run is wrong.
- Each reducer after the first must receive `remaining_budget = max - calls_so_far`.

## Report Contract

Extend report JSON:

```json
{
  "reducers": ["dirs", "files"],
  "phases": [
    {
      "reducer": "dirs",
      "candidate_count": 45,
      "remaining_count": 12,
      "removed_count": 33,
      "guarantee": "complete"
    }
  ],
  "remaining_files": ["relative/file.rs"],
  "removed_files": ["relative/file.rs"],
  "removed_directories": ["relative/dir"]
}
```

Rules:

- Keep existing top-level `candidate_count`, `remaining_count`, and `removed_count` for backward compatibility.
- In directory-only mode, top-level counts refer to files, not directories:
  - `candidate_count`: original file count
  - `remaining_count`: remaining file count
  - `removed_count`: removed file count
- Add `directory_candidate_count`.
- Add `removed_directories`.
- Add `phases`.
- Add `reducers`.
- Best-so-far report uses the same schema with `guarantee: "best-so-far"`.

Reason:

- Users care how many files remain.
- Agents also need to know which directory decisions caused the file removals.

## Trial Tree Policy

Current reusable `trial-current` stays.

Trial tree creation changes from "copy this list of remaining files" to:

```rust
create(fixture_root, work_dir, state, candidate_override)
```

For directory pass:

- `candidate_override` contains the trial directory set.
- Build removed directories from original directory candidates minus remaining candidates.
- Copy files from `original_files` except files under removed directories.

For file pass:

- Start from `state` after directory pass.
- Copy files not removed by directories and not removed by file candidates.

Gotchas:

- Empty directories are still not copied.
- Fixture paths in the trial manifest must stay exact file paths.
- If `fixture.toml` is removed by a directory candidate, fixture discovery will fail. That is `NotInteresting`, not a tool failure, because it is an invalid candidate set for preserving behavior.
- Files under excluded generated dirs are never original candidates and are never copied.

## Directory Reducer Strategy

Do not run DDMin over every directory in the tree at once.

Use top-down depth layers:

1. Collect directory candidates by depth.
2. Start at depth `1`, directly under `fixture-root`.
3. Run DDMin for that depth's active candidates.
4. Apply accepted removals to state.
5. Remove descendants of accepted removed directories from all later layers.
6. Move one depth downward.
7. Skip any directory whose ancestor has already been removed.

Reason:

- A flat list of overlapping directories makes DDMin results harder to reason about.
- Top-down tests broad subtrees first, which is the fastest way to remove copied-project bloat.
- Depth layers turn each DDMin run into mostly disjoint sibling candidates.
- Failed broad directories are then split by descending into children.

Within one depth:

- Candidates are sorted by path.
- DDMin operates on "directories kept".
- Interesting means output still matches when the complement directories are removed.

## Required Verifier Fixture

Add fake-project directory reducer case:

```text
project/
  keep/input.json
  keep/support/a.json
  removable-family/a/noise.txt
  removable-family/b/noise.txt
  removable-family/c/noise.txt
  mixed/keep/input.json
  mixed/remove/noise.txt
```

Command behavior:

- Reads only:
  - `keep/input.json`
  - `keep/support/a.json`
  - `mixed/keep/input.json`
- Ignores other files if present.

Expected directory-only result:

- `fixture3 reduce --reducers dirs --max-oracle-calls 20` removes `removable-family`.
- Does not remove `keep`.
- Does not remove `mixed`.
- May remove `mixed/remove` if budget allows the reducer to descend into `mixed`.

Expected default result:

- `fixture3 reduce --max-oracle-calls 20` uses `dirs,files`.
- Report field `reducers` equals `["dirs", "files"]`.
- At least one phase has `reducer: "dirs"`.

Verifier assertions:

- `fixture3 reduce --reducers dirs --max-oracle-calls 20` exits `0`.
- `removed_directories` contains `removable-family`.
- `remaining_files` includes the three behavior files.
- `work/trials` does not exist.
- `work/trial-current` exists.
- `work/best/reduce-report.json` exists.
- `fixture3 reduce --max-oracle-calls 20` exits `0`.
- Default report includes `reducers: ["dirs", "files"]`.

## Files To Modify

- `apps/fixtures/crates/fixture3/src/args.rs`
- `apps/fixtures/crates/fixture3/src/reduce/candidate.rs`
- `apps/fixtures/crates/fixture3/src/reduce/run.rs`
- `apps/fixtures/crates/fixture3/src/reduce/oracle.rs`
- `apps/fixtures/crates/fixture3/src/reduce/report.rs`
- `apps/fixtures/crates/fixture3/src/reduce/trial_tree.rs`
- `scripts/verify-reducer.py`
- `README.md`
- `.plans/2026-05-13-150929-fixture3-architecture.md.manifest.toml`
- `.plans/2026-05-15-193357-fixture-reducer-adapter.md.manifest.toml`

## Non-Goals

- No source-file internal reduction.
- No TOML key reduction.
- No dependency-edge reduction.
- No parallel oracle execution.
- No committing Guardrail3 L45 into this repo.

## Done Gates

Run:

```bash
python3 scripts/verify-reducer.py
python3 scripts/verify-rust-workspaces.py --fmt
python3 scripts/verify-rust-workspaces.py --check
python3 scripts/verify-rust-workspaces.py --clippy
python3 scripts/verify-rust-workspaces.py --g3rs
scripts/verify-all.sh
git diff --check
```
