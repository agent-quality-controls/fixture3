# Summary

Initialized the standalone `fixture3` repository as a Rust workspace with a minimal `fixture3` binary crate, G3RS policy files, and golden fixture directories.

The architecture plan now lives in this repository and explicitly forbids Rust tests. Verification starts with static checks plus a bootstrap verifier and must switch to self-hosted golden fixtures as soon as the CLI can run a suite.

# Decisions Made

- Used `guardrail3-rs.toml` because the installed `g3rs` validator requires that current marker file at the workspace root.
- Set `[checks] test = false` because this project deliberately bans tests and uses golden behavior fixtures instead.
- Used a real child workspace member at `crates/fixture3` because G3RS rejects a root package workspace member declared as `"."`.
- Added canonical G3RS `clippy.toml`, `deny.toml`, `rustfmt.toml`, and `rust-toolchain.toml` policy surfaces instead of weakening validation.
- Added `scripts/verify-all.sh` as bootstrap verification only. It checks repository shape, the no-test rule, formatting, compilation, clippy, and G3RS rules.

# Key Files For Context

- `.plans/2026-05-13-150929-fixture3-architecture.md`
- `.plans/2026-05-13-150929-fixture3-architecture.md.manifest.toml`
- `guardrail3-rs.toml`
- `scripts/verify-all.sh`
- `crates/fixture3/Cargo.toml`
- `crates/fixture3/src/main.rs`

# Verification

```bash
scripts/verify-all.sh
```

Output:

```text
No findings.
PASS
```

# Next Steps

- Implement the first `fixture3 check` slice against a tiny local fixture suite.
- Replace bootstrap behavior verification with `fixture3 check` once the CLI can run and compare a suite.
