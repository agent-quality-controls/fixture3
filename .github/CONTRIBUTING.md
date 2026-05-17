# Contributing to fixture3

Thanks for your interest. The fastest way to get a change in is usually to open a detailed issue, not a PR.

## How to contribute

### 1. Open a detailed issue (preferred)

When you spot a bug, want a new command, flag, or manifest field, or want to change behavior, open an issue with as much detail as you can:

- The problem (what workflow is awkward or impossible today)
- The proposal (what command, flag, manifest field, or behavior change would solve it)
- Argv shape and exit-code semantics if relevant
- Alternatives considered
- Suggested scope (command, flag, manifest, output format, reducer, doctor)

Detailed issues are easy to one-shot with an agent. The clearer the spec, the faster the turnaround. Most issues get picked up and implemented without needing a PR from you.

The issue forms in `.github/ISSUE_TEMPLATE/` collect these fields by default.

### 2. Send a pull request (also fine)

If you want to implement the change yourself, go ahead. All contributed code must pass the pre-commit hooks and the verification scripts before it can be merged.

## Pre-commit hooks

This repo uses **G3RS** (the Rust variant of [guardrail3](https://github.com/agent-quality-controls/guardrail3)) as a pre-commit gate. G3RS enforces deterministic code quality on every commit: rustfmt formatting, clippy with deny-warnings, dependency allowlists, banned APIs, AST-based source scan, input-validation enforcement, architectural topology, and total suppression visibility.

Hooks run automatically on `git commit`. A failing commit is rejected with a list of findings. Fix them and recommit, or open an issue if a hook fires on code that should be allowed.

## Verification

fixture3 verifies itself with its own approval suite plus extra scripts:

```bash
scripts/verify-all.sh
scripts/verify-fake-project.sh
python3 scripts/verify-reducer.py
```

All three must pass. CI runs them; the pre-commit gate is the local equivalent.

## Dev setup

Requires the Rust toolchain pinned in `rust-toolchain.toml`.

```bash
git clone https://github.com/agent-quality-controls/fixture3.git
cd fixture3
cargo build
```

## Design principles

- **The project owns meaning. fixture3 owns plumbing.** Features and tags do not teach fixture3 what your app does.
- **Fixtures are stable. Approved outputs change.** When code changes, the inputs usually stay put.
- **Review the behavior diff, not the test diff.** Approval files surface what changed in behavior.
- **Fail closed.** Invalid JSON, missing fixtures, schema errors, and command failures exit `2`. Behavior drift exits `1`.
