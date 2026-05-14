# fixture3

## Verification

- Do not add Rust unit tests, integration tests, doc tests, or inline `#[test]` functions.
- Do not add a `tests/` directory.
- Do not use `cargo test` as project verification.
- Use fixture3 behavior suites as the verification model.
- Use `cargo check`, `cargo clippy`, `cargo fmt --check`, and `g3rs validate --rules-only` for compile and static checks.
- As soon as the CLI can run one command, it must verify its own fixture suite through `fixture3 check`.

## G3RS

- Keep `guardrail3-rs.toml` at the repository root.
- Keep `[checks] test = false`.
- Treat any new G3RS failure as a code or architecture failure unless the plan explicitly changes the guardrail.
