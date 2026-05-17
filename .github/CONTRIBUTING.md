# Contributing to fixture3

Thanks for your interest. fixture3 is a CLI for fixture-based approval testing in agent-managed codebases. Contributions that make the approval flow clearer, the manifest schema sharper, or the reducer faster are welcome.

## Dev setup

Requires the Rust toolchain pinned in `rust-toolchain.toml`.

```bash
git clone https://github.com/agent-quality-controls/fixture3.git
cd fixture3
cargo build
```

## Run locally

```bash
cargo run -- --help
cargo run -- doctor
cargo run -- check --all
```

## Verification

fixture3 verifies itself via its own approval suite plus extra scripts:

```bash
scripts/verify-all.sh
scripts/verify-fake-project.sh
python3 scripts/verify-reducer.py
```

All three must pass before a PR is mergeable. `verify-all.sh` covers the tree shape, forbidden files, config, module dependencies, formatting, compilation, clippy, guardrail3 rules, self-hosted fixture behavior, CLI help, and the feature-pipeline contract.

## Design principles

- **The project owns meaning. fixture3 owns plumbing.** Features and tags do not teach fixture3 what your app does - they give stable handles to behavior slices.
- **Fixtures are stable. Approved outputs change.** When code changes, the inputs usually stay put; only the received output moves.
- **Review the behavior diff, not the test diff.** Approval files surface what changed in behavior. That is the review surface.
- **Fail closed.** Invalid JSON, missing fixtures, schema errors, and command failures exit `2`. Behavior drift exits `1`. Only a clean match exits `0`.

## Pull request expectations

- One logical change per PR.
- All three verification scripts pass.
- Approved outputs updated when behavior changes, with a `fixture3 approve` comment explaining why.
- No new lint suppressions, or each is justified in the PR description.
