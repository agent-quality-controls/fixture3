# Summary

Implemented the first manifest-driven `fixture3` V1 command surface and made the repository self-host through `fixture3 check --suite self --manifest fixture3.yaml`.

The verifier stack now reads the architecture manifest, enforces required tree shape, forbidden Rust test surfaces, G3RS config, declared module dependencies, static Rust checks, G3RS, the self golden suite, and the CLI command contract. The self suite now runs nested CLI scenarios for match, mismatch, missing approved output, invalid command exit, invalid JSON output, diff, diff refresh, approve, status, init, fixture hash drift, and normalizer command execution.

# Decisions Made

- Added `lib.rs` as a facade and moved executable stdio handling into `app.rs` because G3RS requires facade-only library roots.
- Added an explicit `cli` feature and gated the facade export because G3RS requires facade exports to be feature-gated.
- Used `serde_norway` instead of `serde_yaml` or `serde_yml` because current RustSec data marks `serde_yml` unsound/unmaintained and `serde_norway` is listed as the maintained fork.
- Implemented `check`, `diff`, `approve`, `status`, and `init` for the V1 surface.
- Implemented optional normalizer command execution through stdin/stdout before JSON normalization.
- Added fail-closed hash comparison when approved metadata exists.
- Enforced module dependencies through manifest rows and `scripts/verify-layer-4-modules.sh` so the small-module crate shape has a mechanical boundary check.
- Replaced the initial weak self fixture with a harness that runs the actual built CLI against nested behavior manifests and records stable behavior summaries.
- Kept approve scenario writes under `.fixture3` so self verification does not mutate committed fixture files.

# Key Files For Context

- `.plans/2026-05-13-150929-fixture3-architecture.md.manifest.toml`
- `.plans/2026-05-13-175011-manifest-driven-check.md`
- `fixture3.yaml`
- `scripts/verify-manifest.py`
- `scripts/self-check-harness.py`
- `scripts/self-case-emit-json.py`
- `scripts/self-case-bad-json.py`
- `scripts/self-case-exit-7.py`
- `scripts/self-case-normalize-json.py`
- `scripts/verify-all.sh`
- `scripts/verify-layer-7-cli.sh`
- `behavior/fixtures/self/cases`
- `behavior/golden/self/approved.normalized.json`
- `behavior/golden/self/approved.meta.json`
- `crates/fixture3/src/app.rs`
- `crates/fixture3/src/manifest.rs`
- `crates/fixture3/src/fixture.rs`
- `crates/fixture3/src/command.rs`
- `crates/fixture3/src/normalize.rs`
- `crates/fixture3/src/storage.rs`
- `crates/fixture3/src/diff.rs`

# Verification

```bash
scripts/verify-all.sh
```

Output:

```text
layer1 tree: PASS
layer2 forbidden: PASS
layer3 config: PASS
layer4 modules: PASS
layer5 static: PASS
layer6 fixture3: PASS
layer7 cli: PASS
PASS
```

# Next Steps

- Add normalizer command execution from the manifest.
- Add explicit override flags for fixture, manifest, and normalizer hash changes.
- Replace structural JSON diff text with a clearer semantic diff format.
