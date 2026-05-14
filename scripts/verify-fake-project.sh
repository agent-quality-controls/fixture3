#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

python3 - <<'PY'
from __future__ import annotations

import sys
import tomllib
from pathlib import Path

manifest = tomllib.loads(Path(".plans/2026-05-14-122624-fake-project-verification.md.manifest.toml").read_text())
missing = [row["path"] for row in manifest.get("tree", []) if not Path(row["path"]).exists()]
if missing:
    for path in missing:
        print(f"missing path: {path}", file=sys.stderr)
    sys.exit(1)
PY

cargo build -q -p fixture3-cli

run_dir="$root/.fixture3/fake-project-run"
rm -rf "$run_dir"
mkdir -p "$(dirname "$run_dir")"
cp -R "$root/examples/fake-project" "$run_dir"

fixture3_bin="$root/target/debug/fixture3"

assert_json() {
  local file="$1"
  local script="$2"
  python3 - "$file" "$script" <<'PY'
from __future__ import annotations

import json
import sys
from pathlib import Path

data = json.loads(Path(sys.argv[1]).read_text())
script = sys.argv[2]
namespace = {"data": data}
exec(script, namespace)
PY
}

cd "$run_dir"

"$fixture3_bin" doctor --manifest fixture3.yaml | grep -q "status: ok"
"$fixture3_bin" explain --suite parser-basic --manifest fixture3.yaml | grep -q "features: parsing"
"$fixture3_bin" explain --suite parser-basic --manifest fixture3.yaml | grep -q "tags: parser,smoke"

"$fixture3_bin" check --suite parser-basic --manifest fixture3.yaml | grep -q "status: matched"
"$fixture3_bin" check --tag smoke --manifest fixture3.yaml --json > check-tag.json
assert_json check-tag.json 'assert [row["suite"] for row in data["suites"]] == ["cli-errors", "parser-basic"]'
assert_json check-tag.json 'assert all(row["status"] == "matched" for row in data["suites"])'

"$fixture3_bin" check --feature parsing --manifest fixture3.yaml --json > check-feature.json
assert_json check-feature.json 'assert [row["suite"] for row in data["suites"]] == ["parser-basic", "cli-errors"]'
assert_json check-feature.json 'assert data["exit_code"] == 0'

"$fixture3_bin" status --feature parsing --manifest fixture3.yaml --json > status-feature.json
assert_json status-feature.json 'assert all(row["approved"] for row in data["suites"])'
assert_json status-feature.json 'assert all(row["received"] for row in data["suites"])'

"$fixture3_bin" diff --suite parser-basic --manifest fixture3.yaml --json > diff-parser.json
assert_json diff-parser.json 'assert data["report"]["status"] == "matched"'

if "$fixture3_bin" check --suite review-drift --manifest fixture3.yaml > review-check.txt; then
  printf 'expected review-drift to differ before approval\n' >&2
  exit 1
fi
grep -q "status: different" review-check.txt

"$fixture3_bin" diff --suite review-drift --manifest fixture3.yaml --refresh --json > review-diff.json || diff_code="$?"
if [[ "${diff_code:-0}" != "1" ]]; then
  printf 'expected review diff refresh exit 1, got %s\n' "${diff_code:-0}" >&2
  exit 1
fi
assert_json review-diff.json 'assert data["report"]["changed"] is True'

"$fixture3_bin" approve --suite review-drift --manifest fixture3.yaml --change behavior/changes/fake-change.md | grep -q "status: approved"
"$fixture3_bin" check --suite review-drift --manifest fixture3.yaml | grep -q "status: matched"

"$fixture3_bin" new suite generated-fake --manifest fixture3.yaml > new-suite.txt
grep -q "generated-fake:" new-suite.txt
test -f behavior/fixtures/generated-fake/example/input.json
test -f behavior/approved/generated-fake/approved.normalized.json

init_dir="$run_dir/init-empty"
mkdir "$init_dir"
cd "$init_dir"
"$fixture3_bin" init | grep -q "status: initialized"
test -f fixture3.yaml

printf 'fake-project: PASS\n'
