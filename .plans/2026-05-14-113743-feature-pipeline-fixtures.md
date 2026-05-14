Goal
- Move this repository from old approved-path wording to fixture3 approval wording.
- Add feature-aware suite management without making fixture3 understand project-specific behavior.
- Keep behavior judgment project-owned: fixture3 manages fixtures, suites, approvals, metadata, and review surfaces.

Approach
- Migrate this repository's approval storage paths from `behavior/approved` and nested `approved` folders to `behavior/approved` and nested `approved` folders.
- Update README, AGENTS, CLI help, init output, self fixtures, and verifier manifests to use fixture/approval terms.
- Extend `fixture3.yaml` with optional suite `tags` and optional top-level `features`.
- Add selector support so `check` and `status` can target `--tag <tag>` or `--feature <feature>` as well as `--suite` and `--all`.
- Add `explain --suite <suite> [--json]` to show resolved fixture globs, command argv, storage paths, tags, feature membership, fixture count, and known files.
- Add `doctor [--json]` to validate manifest shape without running project behavior.
- Add `new suite <name>` to create the standard fixture and approval paths and print the manifest block for that suite.
- Add self fixtures for tag selection, feature selection, explain, doctor, and suite scaffolding.
- Add a feature-pipeline manifest verifier so the new feature contract is checked mechanically.

Key decisions
- Use `approved` for accepted output paths. Fixtures remain inputs; approvals remain reviewed outputs.
- Do not make fixture3 interpret feature semantics. Features only group suites and point at optional spec paths.
- Do not edit manifest YAML in `new suite`; print the manifest block so formatting and project policy stay project-owned.
- Keep `approved_dir`, `approved.normalized.json`, and `approved.meta.json` because approval is the durable output concept.

Files to modify
- `README.md`
- `AGENTS.md`
- `fixture3.yaml`
- `crates/fixture3/src/args.rs`
- `crates/fixture3/src/app.rs`
- `crates/fixture3/src/manifest.rs`
- `crates/fixture3/src/storage.rs`
- `crates/fixture3/src/lib.rs`
- New Rust modules for selection, doctor, and scaffolding.
- `scripts/self-check-harness.py`
- `scripts/verify-all.sh`
- New `scripts/verify-feature-pipeline.py`
- `.plans/2026-05-14-113743-feature-pipeline-fixtures.md.manifest.toml`
- Self fixture manifests and approved self output.

Verification
- `scripts/verify-all.sh`
- `fixture3 check --suite self --manifest fixture3.yaml`
