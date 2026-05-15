Summary
- Added a standalone DDMin package plan separate from fixture reduction.
- The plan records the algorithm contract, scope boundary, implementation references, rejected references, edge cases, and verification model.

Decisions made
- Chose Zeller/Hildebrandt plus Zeller's original Java/Python as the correctness reference.
- Chose Perses `PristineDeltaDebugger` and related list minimizer files as the engineering reference, without copying GPL code.
- Kept `benjholla/ddmin` as a secondary sanity check only because it is old and lightly maintained.
- Kept all fixture-tree, TOML, source-code, manifest, CLI, and proof work out of the standalone DDMin plan.

Key files for context
- `.plans/2026-05-15-145518-ddmin-package.md`
- `.plans/2026-05-14-202459-fixture3-reducer.md`
- `.worklogs/2026-05-15-144738-ddmin-research-stage.md`

Verification
- Read Perses list minimizer source paths through `gh api`.
- Read Perses list minimizer test paths through `gh api`.
- Checked `benjholla/ddmin` repository metadata and file tree.
- `git diff --check`.

Next steps
- Implement only the standalone DDMin module from the new plan.
- Add fixture3 behavior verification for the DDMin edge cases before connecting it to fixture reduction.
