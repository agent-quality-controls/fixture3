Summary
- Added the current DDMin research stage to the reducer plan.
- Recorded the Rust crate and GitHub search scope, rejected candidate repositories, and the current decision to implement only a small generic in-project `ddmin` core after paper verification.

Decisions made
- Rejected all discovered Rust `ddmin` repositories as dependencies because they are unmaintained, unlicensed, GPL-licensed, unpublished, domain-specific, or placeholders.
- Kept `bkushigian/delta-debug` only as a reference shape because it has no license and no maintenance.
- Decided that advanced Perses algorithms are out of scope for the first fixture-tree reducer.
- Kept `treereduce` scoped to optional source-file reduction, not fixture-candidate reduction.

Key files for context
- `.plans/2026-05-14-202459-fixture3-reducer.md`
- `.worklogs/2026-05-15-131610-reducer-research-gate.md`

Verification
- `cargo search ddmin --limit 50`
- `cargo search "delta debugging" --limit 50`
- `cargo search "test case reduction" --limit 50`
- `gh search repos 'ddmin language:Rust' --limit 50`
- `gh search repos 'delta debugging Rust' --limit 100`
- `gh search repos 'test case reduction Rust' --limit 100`
- `gh search repos 'ddmin' --limit 100`
- `gh search repos 'delta-debugging' --limit 100`
- `git diff --check`

Next steps
- Read Zeller and Hildebrandt directly before implementing.
- Write the final `ddmin` input/output contract into the implementation plan after the paper contract is verified.
