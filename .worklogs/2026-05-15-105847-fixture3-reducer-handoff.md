# fixture3 Reducer Handoff

## Current State

- The reducer plan was made decisive after this handoff was first written.
- The current source of truth is `.plans/2026-05-14-202459-fixture3-reducer.md`.
- The embedded plan copy later in this handoff is superseded and must not be used for implementation.
- Repository folder before rename: `/Users/tartakovsky/Projects/websmasher/goldencheck`.
- Project/repo name has already been changed conceptually to `fixture3`; the local folder has not been renamed yet.
- User plans to rename the folder manually, so the next session should rediscover the repo path.
- Working tree currently has an untracked reducer plan:
  - `.plans/2026-05-14-202459-fixture3-reducer.md`
- No reducer implementation has been started.
- No commit has been made for the reducer plan.

## User Persona

- The user is a Harvard IT professor and highly technical reader.
- The user expects precise, researched, defensible information.
- Do not give approximate or unverified claims where verification is needed.
- The user often speaks through imperfect speech-to-text. Treat malformed dictated text as transcription noise, not as a signal about expertise or desired depth.

## Relevant Instructions

- This repo forbids Rust unit tests, integration tests, doc tests, inline `#[test]`, and `tests/`.
- Verification should use fixture behavior, `cargo check`, `cargo clippy`, `cargo fmt --check`, and `g3rs validate --rules-only`.
- Read the 3-5 most recent worklogs at session start before changing files.
- Create a plan before nontrivial code changes.
- Create a worklog before any commit.
- Do not use `cargo test` as project verification.

## Recent Context

- The user wants a `fixture3 reduce` feature.
- The purpose is reducing oversized copied-project fixtures while preserving an explicit behavior contract.
- The guarantee is not global mathematical minimality.
- The guarantee is: no configured reducer pass found removable content under the configured contract.
- The design uses:
  - custom filetree reducer for directory/file deletion
  - `toml_edit` for semantic TOML and `Cargo.toml` reduction
  - `treereduce` plus `tree-sitter-rust` for source-file reduction
  - generated-directory rejection as a hard hygiene gate
  - proof metadata showing what was tried, removed, kept, and changed

## Guardrail3 Walker Review

The user asked whether Guardrail3's `packages/shared/g3-workspace-crawl` tree walker should influence this reducer.

Read these files:

- `/Users/tartakovsky/Projects/websmasher/guardrail3/packages/shared/g3-workspace-crawl/crates/runtime/src/crawl.rs`
- `/Users/tartakovsky/Projects/websmasher/guardrail3/packages/shared/g3-workspace-crawl/crates/runtime/src/recovery.rs`
- `/Users/tartakovsky/Projects/websmasher/guardrail3/packages/shared/g3-workspace-crawl/crates/types/src/entry.rs`
- `/Users/tartakovsky/Projects/websmasher/guardrail3/packages/shared/g3-workspace-crawl/README.md`

Conclusion:

- Do not use Guardrail3 `g3-workspace-crawl` for `fixture3 reduce`.
- Do not use `ignore` as the default reducer walker.
- Guardrail3 crawl is an ingestion snapshot, not a reducer walker.
- It uses `ignore` in phase 1 with gitignore semantics.
- It uses `walkdir` in phase 2 only for targeted recovery.
- It hides banned trees like `.git`, `target`, `node_modules`, `dist`, `.claude/worktrees`, and `behavior/fixtures`.
- For fixture reduction, hidden content is still fixture content and must be seen.
- Generated directories must be detected and rejected, not silently skipped.
- Ignored files must be visible because they may be accidental bloat or intentionally required by the contract.

Decision:

- Use `walkdir` for physical fixture traversal.
- Keep `ignore` only as a possible later dependency if a future contract explicitly needs gitignore-state reporting.

## Dependency Checks Already Done

Checked with `cargo info` and `gh repo view`.

- `treereduce`
  - repo: `https://github.com/langston-barrett/treereduce`
  - stars: 113
  - last push: 2026-05-12
  - license: MIT
  - decision: use for supported source-file reduction

- `tree-sitter-rust`
  - repo: `https://github.com/tree-sitter/tree-sitter-rust`
  - stars: 498
  - last push: 2026-03-27
  - license: MIT
  - decision: use for Rust syntax support through `treereduce`

- `walkdir`
  - repo: `https://github.com/BurntSushi/walkdir`
  - stars: 1501
  - last push: 2024-12-31
  - license: Unlicense OR MIT
  - decision: use for physical fixture traversal

- `ignore`
  - repo: `https://github.com/BurntSushi/ripgrep`
  - stars: 63756
  - last push: 2026-02-27
  - license: Unlicense OR MIT
  - decision: do not use as default filetree walker

- `toml_edit`
  - repo: `https://github.com/toml-rs/toml`
  - stars: 1038
  - last push: 2026-05-14
  - license: MIT OR Apache-2.0
  - decision: use for semantic TOML reduction

- `tempfile`
  - repo: `https://github.com/Stebalien/tempfile`
  - stars: 1429
  - last push: 2026-05-07
  - license: MIT OR Apache-2.0
  - decision: use for candidate workspaces and oracle runs

- `similar`
  - repo: `https://github.com/mitsuhiko/similar`
  - stars: 1267
  - last push: 2026-04-11
  - license: Apache-2.0
  - decision: use only if reducer reports need human-readable diffs beyond existing project diff code

- `sha2`
  - repo: `https://github.com/RustCrypto/hashes`
  - stars: 2215
  - last push: 2026-05-13
  - crate license: MIT OR Apache-2.0
  - decision: use only if existing project hash utilities are insufficient

Rejected:

- `fs_extra`
  - repo: `https://github.com/webdesus/fs_extra`
  - stars: 334
  - last push: 2023-12-13
  - reason: older than one year

- `pathdiff`
  - repo: `https://github.com/Manishearth/pathdiff`
  - stars: 66
  - last push: 2025-02-25
  - reason: below 100 stars and older than one year

- `evaporust`
  - reason: published crate is effectively a placeholder

## Current Reducer Plan

Read `.plans/2026-05-14-202459-fixture3-reducer.md`. It is the source of truth for `fixture3 reduce`. The older embedded `fixture3 minimize` plan was removed from this handoff to prevent stale implementation instructions.

## Next Session Startup

After folder rename:

1. Locate the renamed repo.
2. Read this handoff.
3. Read `.plans/2026-05-14-202459-fixture3-reducer.md`.
4. Read the latest 3-5 `.worklogs`.
5. Run `git status --short`.
6. Decide whether to convert the reducer plan into a manifest before implementation.
7. Do not start implementation without a fresh plan/manifest if scope remains this large.
