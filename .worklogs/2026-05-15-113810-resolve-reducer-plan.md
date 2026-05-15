Summary
- Rewrote the reducer plan to remove open implementation choices and make `fixture3 minimize` executable by future agents.
- Marked the older embedded handoff plan copy as superseded.
- Included the root `code-sessions` resume helper in the working set.

Decisions made
- Chose in-place mutation as normal reducer behavior and dry-run restore behavior as the non-mutating mode.
- Chose JSON row matching for stdout contracts and stderr text exclusion.
- Chose manifest-owned proof path, source globs, pass list, and exit-code semantics.
- Chose `walkdir`, `toml_edit`, `treereduce`, `tree-sitter-rust`, `tempfile`, `similar`, and existing `sha2` as the concrete dependency stack.

Key files for context
- `.plans/2026-05-14-202459-fixture3-reducer.md`
- `.worklogs/2026-05-15-105847-fixture3-reducer-handoff.md`
- `code-sessions`

Verification
- `rg` scan for unresolved open-decision language in the reducer plan.
- `git diff --check`.

Next steps
- Convert the decisive reducer plan into a manifest before implementation.
