# Reducer Runtime Controls Plan

## Goal

Make `fixture3 reduce` usable on real copied fixture trees by adding:

1. `--max-oracle-calls`
2. best-so-far report writing during the run
3. one reusable trial directory instead of one directory per oracle call

This plan does not change the DDMin algorithm. It changes the fixture3 reducer adapter around DDMin.

## Decisions

### `--max-oracle-calls`

Add:

```bash
fixture3 reduce --max-oracle-calls <count>
```

Behavior:

- The flag is optional.
- Omitted means no limit and keeps current complete-proof behavior.
- `0` is rejected by clap through `NonZeroUsize`.
- The value is passed directly to `fixture3_ddmin::DdminOptions`.
- When the limit is reached, the final report uses the existing guarantee string:

```text
incomplete:max-oracle-calls-reached
```

Reason:

- The DDMin crate already supports `max_oracle_calls`.
- The reducer should not implement its own stop condition outside the algorithm.
- Unlimited remains available for small fixtures where a complete proof is wanted.

Files:

- `apps/fixtures/crates/fixture3/src/args.rs`
- `apps/fixtures/crates/fixture3/src/reduce/run.rs`

### Best-so-far Report

Write these files after the baseline trial and after every `Interesting` oracle result:

```text
<work-dir>/best/reduce-report.json
<work-dir>/best/removed-files.txt
<work-dir>/best/remaining-files.txt
```

The existing final files remain:

```text
<work-dir>/reduce-report.json
<work-dir>/removed-files.txt
<work-dir>/remaining-files.txt
```

Best report schema:

```json
{
  "suite": "name",
  "fixture_root": "path",
  "work_dir": "path",
  "candidate_count": 303,
  "remaining_count": 172,
  "removed_count": 131,
  "oracle_calls": 758,
  "interesting_trials": 17,
  "not_interesting_trials": 741,
  "unresolved_trials": 0,
  "guarantee": "best-so-far",
  "remaining_files": ["relative/path"],
  "removed_files": ["relative/path"]
}
```

Behavior:

- `best-so-far` is an interim guarantee string only.
- Final report keeps the DDMin guarantee string.
- If no `Interesting` trial has happened, `best/` is absent.
- The baseline full candidate set counts as `Interesting` when it matches approved output.
- A caller can interrupt the process and still inspect the best known reduction.

Reason:

- DDMin only returns final output after the loop completes or hits a limit.
- The oracle is the only place that observes each successful trial in real time.
- The report writer already knows how to serialize file lists, so this should reuse the report module.

Files:

- `apps/fixtures/crates/fixture3/src/reduce/oracle.rs`
- `apps/fixtures/crates/fixture3/src/reduce/report.rs`
- `apps/fixtures/crates/fixture3/src/reduce/run.rs`

### Reusable Trial Directory

Replace per-call directories:

```text
<work-dir>/trials/<call-number>/
```

with one reusable directory:

```text
<work-dir>/trial-current/
```

Each oracle call:

1. Removes `<work-dir>/trial-current`.
2. Recreates `<work-dir>/trial-current/fixture-root`.
3. Copies only the remaining candidate files.
4. Writes `<work-dir>/trial-current/manifest.fixture3.yaml`.
5. Uses received and diff dirs under `<work-dir>/trial-current`.

Keep optional debug output out of this change. There is no retained trial history.

Reason:

- The Guardrail3 probe produced 829 trial directories and 351 MB.
- The reducer only needs one trial tree to evaluate the next candidate set.
- Best-so-far file lists are enough to reconstruct a reduced tree from the original fixture root.

Files:

- `apps/fixtures/crates/fixture3/src/reduce/trial_tree.rs`
- `apps/fixtures/crates/fixture3/src/reduce/oracle.rs`

## Implementation Steps

1. Add `max_oracle_calls: Option<NonZeroUsize>` to `ReduceArgs`.
2. Add help text for `--max-oracle-calls`, `best/reduce-report.json`, and `trial-current`.
3. Pass `args.max_oracle_calls.map(NonZeroUsize::get)` into `DdminOptions`.
4. Extend `ReduceOracle` with:
   - original ordered candidate list
   - counters for interesting, not-interesting, unresolved
   - best remaining candidate list
   - reducer args needed for report writing
5. After every oracle evaluation, update counters.
6. When outcome is `Interesting`, write `<work-dir>/best/*`.
7. Change `trial_tree::create` to use `<work-dir>/trial-current` and remove the `call_number` directory layout.
8. Keep `call_number` only as an in-memory counter and report field.
9. Update `scripts/verify-reducer.py`:
   - run one reducer case with `--max-oracle-calls 2`
   - assert final guarantee is `incomplete:max-oracle-calls-reached`
   - assert `<work-dir>/best/reduce-report.json` exists
   - assert there is no `<work-dir>/trials`
   - assert `<work-dir>/trial-current` exists
10. Update README command docs.
11. Update architecture manifest and reducer plan manifest.

## Verification

Run:

```bash
python3 scripts/verify-reducer.py
scripts/verify-all.sh
git diff --check
```

Expected reducer verifier additions:

- `--max-oracle-calls` is present in `fixture3 reduce --help`.
- Interrupted-by-limit reducer run exits `0`.
- Final report guarantee is `incomplete:max-oracle-calls-reached`.
- Best report exists and contains a non-empty remaining file list.
- Work dir contains `trial-current`.
- Work dir does not contain `trials`.

## Non-Goals

- No parallel oracle execution.
- No text-mode output.
- No automatic timeout.
- No source-code or TOML reduction.
- No permanent Guardrail3 fixture import into this repository.
