# Goal

Design `spec3`: a Rust CLI and library for spec-driven development where a reviewed machine-readable spec becomes the implementation contract.

`spec3` is not a fixture/golden-output tool. Fixture tools catch behavior drift after implementation. `spec3` catches plan-to-code drift while implementation is being built.

The core invariant:

```text
prose plan -> reviewed spec -> locked spec -> implementation -> deterministic conformance check
```

# Problem

Current manifest-driven development is too easy to weaken.

Observed failure mode:

```text
agent writes plan
agent writes manifest
agent writes verifier script
agent edits code
agent edits manifest/verifier when they block
verifier passes
agent claims done
```

That only proves the current verifier agrees with current code. It does not prove the implementation matches the reviewed plan.

`spec3` must make contract changes explicit by locking the spec and verifier files before implementation verification.

# Product Name

Use package and binary name:

```text
spec3
```

# Format

Support both:

- JSONC source specs: `.spec3.jsonc`
- Plain JSON source specs: `.spec3.json`

Plain JSON is parsed directly. JSONC is parsed with a real JSONC parser, not regex. Current preferred Rust dependency is `jsonc-parser`, then typed deserialization through `serde`.

The locked machine contract is canonical strict JSON.

Pipeline:

```text
.spec3.jsonc or .spec3.json
-> parse
-> typed spec3 model
-> canonical JSON
-> hash canonical JSON
-> write/read lock
-> verify implementation
```

Comments are allowed only in JSONC and must not affect the canonical contract hash.

# Source And Lock Files

Suggested files:

```text
.plans/foo.md
.plans/foo.spec3.jsonc
.plans/foo.spec3.lock.json
```

The lock records:

- plan path
- plan hash
- source spec path
- canonical spec hash
- verifier command list
- verifier file hashes
- spec3 version
- lock created time

# Command Surface

## `spec3 lint`

Validate a source spec without locking or running implementation checks.

Checks:

- source parses as JSON or JSONC
- typed spec model is valid
- requirement IDs are unique
- requirement kinds are known
- every requirement has a verifier strategy
- referenced verifier commands are declared
- referenced paths are normalized

Example:

```bash
spec3 lint .plans/foo.spec3.jsonc
```

## `spec3 lock`

Freeze the reviewed contract.

Checks:

- `lint` passes
- plan exists
- verifier files exist
- verifier command declarations are valid

Writes:

```text
.plans/foo.spec3.lock.json
```

Example:

```bash
spec3 lock .plans/foo.spec3.jsonc
```

## `spec3 verify`

Verify code against the locked contract.

Checks before running verifiers:

- plan hash matches lock
- canonical spec hash matches lock
- verifier file hashes match lock
- verifier command list matches lock

Then runs the declared verifier commands.

Example:

```bash
spec3 verify .plans/foo.spec3.jsonc
spec3 verify .plans/foo.spec3.lock.json
```

Exit codes:

- `0`: locked contract matches and all verifiers pass
- `1`: implementation does not satisfy spec
- `2`: spec, lock, verifier, parser, or runtime error

## `spec3 status`

Show whether the spec can be trusted right now.

Reports:

- plan drift
- spec drift
- verifier drift
- missing lock
- latest verifier status if available

Example:

```bash
spec3 status .plans/foo.spec3.jsonc
```

## `spec3 normalize`

Emit canonical strict JSON from JSONC or JSON.

This is useful for debugging lock diffs and for generated tooling.

Example:

```bash
spec3 normalize .plans/foo.spec3.jsonc
```

# Requirement Model

Use a typed top-level `requirements` object instead of a loose array of ad hoc block names.

Draft shape:

```jsonc
{
  "version": 1,
  "plan": ".plans/foo.md",
  "requirements": {
    "tree": [],
    "text": [],
    "dependencies": [],
    "exports": [],
    "closedSets": [],
    "schemas": [],
    "cli": [],
    "fixtures": [],
    "commands": []
  },
  "verifiers": []
}
```

Every requirement has:

- `id`
- `kind`
- optional `reason`
- kind-specific fields
- verifier ownership, either direct or implied by kind

# Standard Requirement Categories

## `tree`

Universal file tree contracts.

Use cases:

- required files/directories
- forbidden files/directories
- partial tree shape where unspecified paths are allowed

Example:

```jsonc
{
  "id": "TREE_CLI",
  "kind": "tree",
  "reason": "The CLI crate must keep stable entry modules.",
  "required": {
    "crates": {
      "spec3": {
        "files": ["Cargo.toml"],
        "dirs": {
          "src": {
            "files": ["main.rs", "args.rs", "spec.rs", "lock.rs"]
          }
        }
      }
    }
  },
  "forbidden": ["tests", "**/*_tests.rs"]
}
```

Verifier implementation:

- use a filesystem crawler
- normalize paths
- check required subtree exists
- check forbidden globs do not match

## `text`

Universal required/forbidden text or pattern contracts.

Use cases:

- no `cargo test`
- no `#[test]`
- required generated marker
- forbidden old package name

This should start with fixed-string matching. Pattern matching should be explicit and likely limited to a safe glob-like syntax, not regex by default.

## `dependencies`

Package, import, and module dependency contracts.

Use cases:

- required Cargo dependency with path/features
- forbidden Cargo dependency
- forbidden TypeScript import
- allowed module dependency edge
- forbidden module dependency edge

This category is partially language-specific.

Verifier implementation:

- Rust: parse `Cargo.toml`, possibly use cargo metadata
- TypeScript: parse package metadata and imports
- generic module graph: project-owned extractor can emit dependency facts for spec3 comparison

## `exports`

Exported API contracts.

Use cases:

- required exported type/function/constant
- forbidden exported type/function/constant
- closed exported surface compared to an inventory file

This replaces the earlier name `publicSurface`.

Verifier implementation:

- Rust: use rustdoc JSON, `cargo public-api`, or a Rust AST extractor
- TypeScript: use TypeScript compiler API or generated declaration files
- spec3 should compare extracted facts, not parse every language itself in V1

## `closedSets`

Finite value contracts.

Use cases:

- enum variants
- rule IDs
- status strings
- exit codes
- output kind strings
- command names

Verifier implementation:

- language-specific extractor for code-owned sets
- config/schema parser for data-owned sets
- exact set comparison when `mode = "closed"`
- subset comparison when `mode = "required"`

## `schemas`

Structured data contracts.

Use cases:

- SQL table columns, indexes, foreign keys, checks
- JSON schema files
- config schema fields
- parser model fields
- DTO fields

Verifier implementation:

- SQL parser where available
- JSON Schema validator for JSON-shaped contracts
- language-specific parser model extractor where needed

## `cli`

CLI surface contracts.

Use cases:

- subcommands
- flags
- mutual exclusions
- required one-of groups
- help text fragments
- exit codes
- JSON output fields

Verifier implementation:

- run `--help`
- run command cases where needed
- parse JSON output for fields
- compare exit codes

## `fixtures`

Fixture wiring contracts, not golden behavior comparison.

Use cases:

- fixture suite exists
- fixture files are present
- fixture manifest references the expected suite
- coverage matrix has entries for required rules

This category must not duplicate `fixture3`. It verifies that fixture infrastructure exists and is wired. It does not decide whether current behavior matches approved output.

## `commands`

External command contracts.

Use cases:

- `cargo check`
- `cargo clippy`
- `g3rs validate`
- project-owned verifier scripts
- `fixture3 check`

Commands are the final mechanical gates. They are also lock inputs when they reference project-owned scripts.

# Verifier Model

Specs declare verifiers separately from requirements.

Draft:

```jsonc
{
  "verifiers": [
    {
      "id": "tree",
      "command": ["spec3", "builtin", "tree"],
      "files": []
    },
    {
      "id": "rust-exports",
      "command": ["scripts/verify-rust-exports.sh"],
      "files": ["scripts/verify-rust-exports.sh"]
    }
  ]
}
```

Built-in verifiers should exist for universal categories:

- `tree`
- `text`
- maybe `commands`

Language-specific and project-specific verifiers can be external commands.

# Built-In vs External Verifiers

V1 should include only high-confidence built-ins:

- tree required/forbidden checks
- fixed-string required/forbidden text checks
- command execution with exit-code checking
- spec/lock/hash validation

Do not rush language parsers into V1. Rust/TypeScript exports and dependency graphs can begin as external verifier commands that emit normalized JSON facts.

# Output Contract

Human output should be compact:

```text
spec: .plans/foo.spec3.jsonc
lock: matched
requirements: 12
verifiers: 4
status: passed
```

JSON output should be available for agents and CI:

```bash
spec3 verify .plans/foo.spec3.jsonc --json
```

# Hashing Rules

Hash these:

- prose plan bytes
- canonical parsed spec JSON
- verifier files listed in the spec
- spec3 binary version in the lock

Do not include JSONC comments in the canonical spec hash.

Open decision:

- whether to also store `sourceHash` for the original JSONC bytes so comment-only changes can be reported as non-contract drift.

# Relationship To Existing Tools

`spec3` complements:

- `fixture3`: behavior and output drift
- `g3rs` / `g3ts`: static architecture and style guardrails
- project-owned scripts: language-specific extraction and conformance checks

`spec3` should not replace:

- fixture/golden testing
- static lint tools
- project-specific verifiers
- human/agent spec authoring

# JSONC Dependency Decision

Use `jsonc-parser` unless implementation research finds a concrete blocker.

Rejected:

- hand-written comment stripping
- regex comment stripping
- YAML as canonical format
- JSON5 as canonical format

Reason:

- JSONC gives comments with minimal syntax expansion.
- JSON canonicalization stays straightforward.
- JSON Schema and Serde models stay usable.

# Files To Modify For First Implementation

If implemented in this repository:

- `Cargo.toml`
- `crates/spec3/Cargo.toml`
- `crates/spec3/src/main.rs`
- `crates/spec3/src/args.rs`
- `crates/spec3/src/spec.rs`
- `crates/spec3/src/jsonc.rs`
- `crates/spec3/src/canonical.rs`
- `crates/spec3/src/lock.rs`
- `crates/spec3/src/hash.rs`
- `crates/spec3/src/verify.rs`
- `crates/spec3/src/builtin/tree.rs`
- `crates/spec3/src/builtin/text.rs`
- `crates/spec3/src/builtin/commands.rs`
- `behavior/fixtures/spec3/`
- `fixture3.yaml`

Open decision:

- whether `spec3` belongs in this repository temporarily or should start as a separate `/Users/tartakovsky/Projects/websmasher/spec3` repository.

# V1 Definition Of Done

- `spec3 lint` validates JSON and JSONC source specs.
- `spec3 normalize` emits canonical strict JSON.
- `spec3 lock` writes a lock with plan, spec, verifier hashes.
- `spec3 verify` refuses to run when plan/spec/verifier drift exists.
- `spec3 verify` runs built-in tree/text/command checks.
- The repo has at least one self fixture checked through `fixture3`.
- No verifier script is allowed to silently define the spec shape. The spec owns the contract; verifiers implement checks.

