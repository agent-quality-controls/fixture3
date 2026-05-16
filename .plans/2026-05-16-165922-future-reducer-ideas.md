# Future Reducer Ideas

## Status

Park these ideas until a real fixture proves that `dirs,files` is not enough.

Current practical reducer:

```text
fixture3 reduce --reducers dirs,files
```

Current behavior:

- `dirs` removes physical directory subtrees top-down.
- `files` does final individual file cleanup.
- Default budgets are `dirs=200` and `files=100`.
- This is enough for current needs.

## Ideas

### path-families

Group paths by common semantic names and reduce whole groups.

General default families:

```text
tests
test_support
rule_tests
benches
examples
docs
fixtures
mocks
snapshots
generated
```

Why it might matter:

- Directory reduction removes one physical subtree at a time.
- Path-family reduction can remove same-purpose paths across several subtrees in one candidate.
- It does not need file editing.

Risk:

- Family rules can become project-specific.
- Defaults must stay generic.

### configurable path families

Let projects define extra family names in fixture3 config.

Example project-specific families:

```text
advisories
licenses
sources
bans
```

Why it might matter:

- Some codebases have domain-specific rule families spread across many directories.
- A project can name those families without fixture3 hardcoding them.

Risk:

- This adds manifest surface area.
- It should only be added after a real fixture needs it.

### packages

Reduce whole Cargo workspace packages.

Candidate operation:

- Remove a package directory.
- Remove matching `workspace.members` entries.
- Remove path dependency entries that point to the package.
- Remove workspace dependency entries if they become unused.

Why it might matter:

- Copied fixtures from monorepos may contain whole irrelevant crates.
- Package-level reduction removes coherent code units.

Risk:

- Requires TOML editing.
- Invalid Cargo states are easy to create.
- Needs `toml_edit`.
- More complex than current needs.

### toml

Reduce TOML keys, tables, arrays, workspace members, dependency entries, and config rows.

Why it might matter:

- TOML rows can keep otherwise-removable code alive through references.
- TOML reduction can unlock package or module deletion.

Risk:

- It usually removes little code by itself.
- It is an unlocker, not the main reducer.
- Must use `toml_edit`, not string editing.

### rust-modules

Remove Rust module declarations together with matching module files or directories.

Candidate operation:

- Remove `mod foo;`.
- Remove `foo.rs` or `foo/mod.rs`.
- Remove the nested subtree if the behavior still matches.

Why it might matter:

- Directory deletion can fail when `lib.rs` or `mod.rs` still references the removed directory.
- Module-aware deletion can remove source code that raw directory deletion cannot remove.

Risk:

- Rust module layout has many edge cases.
- Needs parser-aware handling.
- More complex than current needs.

### code

Reduce contents of remaining large source files.

Possible tool:

- `treereduce`
- another syntax-aware reducer

Why it might matter:

- After directory and package reduction, a few large files may remain.
- Source reduction reduces line count inside those files.

Risk:

- It does not remove whole files or directories.
- Lower impact on agent context than subtree/package reduction.

### hygiene gate

Fail or report generated/vendor content inside fixtures.

Forbidden or suspicious directories:

```text
.git
target
node_modules
dist
.fixture3
.goldencheck
```

Why it might matter:

- Keeps fixtures clean before reduction starts.
- Prevents copied build output from becoming fixture input.

Risk:

- This is fixture-quality enforcement, not a reducer.
- It should be separate from DDMin reduction.

## Priority If Reopened

Build only when a real fixture shows `dirs,files` leaves too much code.

Order:

```text
path-families
configurable path families
packages
toml
rust-modules
code
hygiene gate
```

Do not build all of these speculatively.
