Goal
- Add a fake project that exercises the fixture3 feature-pipeline workflow end to end.
- Verify doctor, explain, check by suite/tag/feature, status JSON, diff JSON, approve, and new suite.

Approach
- Create `examples/fake-project` with a small deterministic JSON-producing script, fixtures, approvals, changes, and `fixture3.yaml`.
- Add `scripts/verify-fake-project.sh` that copies the fake project to `.fixture3/fake-project-run` and runs fixture3 commands against the copy.
- Include the fake-project verifier in `scripts/verify-all.sh` so this workflow stays checked.
- Keep approval-changing commands on the disposable copy, not the committed example.

Key decisions
- Use a shell script command in the fake project because it is easy to inspect and does not add dependencies.
- Keep the fake project outside `behavior/` so it reads like a user project, not an internal self fixture.
- Use `cargo run -p fixture3-cli --` from the verifier so it checks the current source binary.

Files to modify
- `examples/fake-project/fixture3.yaml`
- `examples/fake-project/scripts/fake-app.sh`
- `examples/fake-project/behavior/fixtures/**`
- `examples/fake-project/behavior/approved/**`
- `examples/fake-project/behavior/changes/.gitkeep`
- `scripts/verify-fake-project.sh`
- `scripts/verify-all.sh`

Verification
- `scripts/verify-fake-project.sh`
- `scripts/verify-all.sh`
