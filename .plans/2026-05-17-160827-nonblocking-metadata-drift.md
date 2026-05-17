# Nonblocking Metadata Drift

Goal
- `fixture3 check` must not fail only because approved metadata has old fixture or manifest hashes.
- Fixture and manifest edits are valid inputs to the approval workflow.
- `check` must always write current received output and current received metadata after the suite command succeeds.

Approach
- Change the existing `hash-drift` self fixture to expect successful check output instead of a tool error.
- Remove the blocking approved-versus-received metadata hash assertion from `storage::write_received`.
- Remove the now-unused metadata hash assertion helper.
- Regenerate the self approved output.
- Run fixture and static verification.

Files
- `scripts/self-check-harness.py`
- `apps/fixtures/crates/fixture3/src/storage.rs`
- `apps/fixtures/crates/fixture3/src/metadata.rs`
- `behavior/approved/self/approved.normalized.json`
- `behavior/approved/self/approved.meta.json`
