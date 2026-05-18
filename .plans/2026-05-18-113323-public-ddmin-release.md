# Public fixture3-ddmin Release

Goal
- Make fixture3-ddmin a standalone public Rust crate that can be published from main after PR CI passes.
- Keep all implementation work on development.
- Add enough README, rustdoc, examples, package metadata, and CI checks that agents can use and verify the crate without private context.

Approach
- Update package metadata so packages/ddmin/crates/ddmin can publish to crates.io.
- Add an exhaustive crate README focused on agent usage and human usage.
- Add rustdoc to every public API item and method.
- Add examples covering minimal use, budget limits, unresolved outcomes, and file/tree-style reduction.
- Update manifest verifier expectations from private package to publishable package.
- Add release metadata, docs.rs metadata, controlled package include rules, and repo-root CI steps for ddmin docs, verifier, and publish dry run.
- Run local verification, commit to development, push development, open PR to main, wait for CI, merge if allowed, then publish from main.

Files
- packages/ddmin/crates/ddmin/Cargo.toml
- packages/ddmin/crates/ddmin/README.md
- packages/ddmin/crates/ddmin/src/lib.rs
- packages/ddmin/crates/ddmin/src/types.rs
- packages/ddmin/crates/ddmin/src/algorithm.rs
- packages/ddmin/crates/ddmin/examples/*.rs
- packages/ddmin/release-plz.toml
- packages/ddmin/cliff.toml
- .plans/2026-05-15-145518-ddmin-package.md.manifest.toml
- packages/ddmin/scripts/verify-ddmin.py
- .github/workflows/ci.yml
