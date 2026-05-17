# fixture3

[![crates.io](https://img.shields.io/crates/v/fixture3)](https://crates.io/crates/fixture3)
[![license](https://img.shields.io/github/license/agent-quality-controls/fixture3)](LICENSE)
[![rust](https://img.shields.io/badge/rust-stable-orange)](rust-toolchain.toml)
[![ci](https://img.shields.io/github/actions/workflow/status/agent-quality-controls/fixture3/ci.yml?branch=main&label=ci)](https://github.com/agent-quality-controls/fixture3/actions/workflows/ci.yml)
[![issues](https://img.shields.io/github/issues/agent-quality-controls/fixture3)](https://github.com/agent-quality-controls/fixture3/issues)

`fixture3` is a CLI for fixture-based approval testing in agent-managed codebases.

## Install

```bash
cargo install cargo-binstall
cargo binstall fixture3
fixture3 --version
```

Use `cargo binstall fixture3` as the install path. The crates.io package is an install stub for cargo-binstall metadata, not the real CLI implementation.

## Quick start

```bash
fixture3 init
fixture3 doctor
fixture3 check --all
```

The full agent guide is in `fixture3 --help`. It covers the model, manifest schema, fixture substitution, files, feature selectors, JSON output, approval flow, and exit codes from the top-level help screen.

## More

- [Philosophy](https://github.com/agent-quality-controls/fixture3/wiki/Philosophy) — fixtures vs snapshots, fail-closed semantics, why DDMin reduction.
- [Comparison](https://github.com/agent-quality-controls/fixture3/wiki/Comparison) — fixture3 vs insta, ApprovalTests, expect-test, cargo-insta.
- [The model](https://github.com/agent-quality-controls/fixture3/wiki/The-Model) — fixtures, suites, features, tags, approved and received output.
- [Manifest](https://github.com/agent-quality-controls/fixture3/wiki/Manifest) — `fixture3.yaml` schema with examples.
- [Files](https://github.com/agent-quality-controls/fixture3/wiki/Files) — committed vs generated paths.
- [Workflow](https://github.com/agent-quality-controls/fixture3/wiki/Workflow) — init, check, diff, approve, status, reduce.
- [Commands](https://github.com/agent-quality-controls/fixture3/wiki/Commands) — full CLI reference, exit codes, fail-closed checks.
- [Agent output](https://github.com/agent-quality-controls/fixture3/wiki/Agent-Output) — JSON output for every command.
- [Verification](https://github.com/agent-quality-controls/fixture3/wiki/Verification) — repository self-verification scripts.
- [Thanks](https://github.com/agent-quality-controls/fixture3/wiki/Thanks) — DDMin (Zeller and Hildebrandt), ApprovalTests, insta.
- [Contributing](.github/CONTRIBUTING.md) — open a detailed issue first; PRs must pass the G3RS pre-commit gate and the verification scripts.

---

Part of [Agent Quality Controls](https://github.com/agent-quality-controls).

## License

MIT
