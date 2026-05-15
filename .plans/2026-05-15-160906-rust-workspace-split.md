# Rust Workspace Split Plan

## Goal

Convert the repository from one root Rust workspace into separate adopted Rust workspaces.

The repo root remains a coordination root for docs, plans, scripts, GitHub workflows, and fixture assets.

## Target Workspaces

```text
apps/fixtures/
apps/fixture3-install/
packages/ddmin/
```

Each workspace has:

- `Cargo.toml` with `[workspace]`
- `guardrail3-rs.toml`
- `clippy.toml`
- `deny.toml`
- `rustfmt.toml`

The repo root must not have a Rust adoption marker pair.

## Move Map

```text
crates/fixture3/* -> apps/fixtures/crates/fixture3/*
crates/fixture3-install/* -> apps/fixture3-install/crates/fixture3-install/*
packages/ddmin/* -> packages/ddmin/crates/ddmin/*
```

## Root Files

Remove root Rust workspace ownership:

- delete root `Cargo.toml`
- delete root `guardrail3-rs.toml`

Keep root shared verification assets:

- `scripts/`
- `.plans/`
- `.worklogs/`
- `.githooks/`
- `.github/`
- `README.md`
- `fixture3.yaml`
- behavior fixtures

## Workspace Config

Use the same strict Rust lint policy in every workspace.

Only disable the G3RS test family:

```toml
[checks]
test = false
```

Do not disable `garde`, `release`, `hooks`, or any other G3RS family.

## Script Updates

Update root verification scripts to run package commands from the correct workspace paths.

Required command changes:

- `cargo run -p fixture3-cli` becomes `cargo run --manifest-path apps/fixtures/Cargo.toml -p fixture3-cli`
- `cargo run -p fixture3-ddmin` becomes `cargo run --manifest-path packages/ddmin/Cargo.toml -p fixture3-ddmin`
- root `cargo fmt/check/clippy` becomes per-workspace loops over the three workspace roots

## G3RS Verification

`g3rs validate-repo` must pass.

Each workspace must pass:

```bash
g3rs validate --path apps/fixtures --rules-only
g3rs validate --path apps/fixture3-install --rules-only
g3rs validate --path packages/ddmin --rules-only
```

## Done State

- root has no Rust workspace marker pair
- all three target workspaces exist
- `g3rs validate-repo` passes
- all three workspace G3RS validations pass
- root `scripts/verify-all.sh` passes
- no Rust tests are added
