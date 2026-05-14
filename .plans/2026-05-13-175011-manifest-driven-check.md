# Goal

Implement the first self-hosting `fixture3 check` slice with manifest-driven verification.

The repository should have a machine-readable manifest that describes required files, forbidden test surfaces, allowed module dependencies, required CLI commands, expected generated files, and verification commands. The verifier scripts must read that manifest and decide whether the repository satisfies it.

# Approach

1. Expand `.plans/2026-05-13-150929-fixture3-architecture.md.manifest.toml` with CLI modules, allowed module dependencies, `check` command expectations, generated output paths, and self-hosting command metadata.
2. Replace the monolithic bootstrap verifier with layered scripts that read the manifest:
   - `scripts/verify-layer-1-tree.sh`
   - `scripts/verify-layer-2-forbidden.sh`
   - `scripts/verify-layer-3-config.sh`
   - `scripts/verify-layer-4-modules.sh`
   - `scripts/verify-layer-5-static.sh`
   - `scripts/verify-layer-6-fixture3.sh`
   - `scripts/verify-manifest.py`
3. Implement one CLI command: `fixture3 check --suite <name> --manifest <path>`.
4. Add a minimal self-hosting fixture suite in `fixture3.yaml`.
5. Make `scripts/verify-all.sh` run every verifier layer, including `fixture3 check`.

# Key Decisions

- Implement only `check` now. `diff`, `approve`, `status`, and `init` stay out of scope until the first behavior loop is proven.
- Keep modules small, but enforce imports through manifest-declared `[[module_dep]]` rows.
- Use a Python verifier for manifest inspection because the repository already uses Python for TOML checks and it avoids fragile shell parsing.
- Use JSON normalization in the CLI for V1. A normalizer command can be added later from the architecture plan.
- Use one deterministic self command script to prove fixture discovery, command execution, storage, diffing, and exit codes.

# Files To Modify

- `.plans/2026-05-13-150929-fixture3-architecture.md.manifest.toml`
- `Cargo.toml`
- `crates/fixture3/Cargo.toml`
- `crates/fixture3/src/*.rs`
- `fixture3.yaml`
- `behavior/fixtures/self/basic/input.txt`
- `behavior/golden/self/approved.normalized.json`
- `behavior/golden/self/approved.meta.json`
- `scripts/verify-all.sh`
- `scripts/verify-layer-*.sh`
- `scripts/verify-manifest.py`
- `scripts/self-emit-json.py`
