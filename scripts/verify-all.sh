#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

required_paths=(
  "Cargo.toml"
  "crates/goldencheck/Cargo.toml"
  "crates/goldencheck/src/main.rs"
  "guardrail3-rs.toml"
  "clippy.toml"
  "deny.toml"
  "behavior/fixtures/bootstrap/.gitkeep"
  "behavior/golden/bootstrap/.gitkeep"
  "behavior/changes/.gitkeep"
  "scripts/verify-all.sh"
)

for required_path in "${required_paths[@]}"; do
  if [[ ! -e "$required_path" ]]; then
    printf 'FAIL missing path: %s\n' "$required_path"
    exit 1
  fi
done

if find . -path './target' -prune -o -path './.git' -prune -o -path './tests' -print | grep -q .; then
  printf 'FAIL forbidden tests directory exists\n'
  exit 1
fi

if rg --fixed-strings '#[test]' crates Cargo.toml >/dev/null; then
  printf 'FAIL forbidden Rust test attribute found\n'
  exit 1
fi

if rg --fixed-strings '#[cfg(test)]' crates Cargo.toml >/dev/null; then
  printf 'FAIL forbidden Rust cfg(test) attribute found\n'
  exit 1
fi

if rg --fixed-strings 'cargo test' crates Cargo.toml >/dev/null; then
  printf 'FAIL forbidden cargo test reference found\n'
  exit 1
fi

while IFS= read -r -d '' script_path; do
  if rg --fixed-strings 'cargo test' "$script_path" >/dev/null; then
    printf 'FAIL forbidden cargo test reference found in %s\n' "$script_path"
    exit 1
  fi
done < <(find scripts -type f ! -name verify-all.sh -print0)

python3 - <<'PY'
from pathlib import Path
import sys
import tomllib

config = tomllib.loads(Path("guardrail3-rs.toml").read_text())
if config.get("checks", {}).get("test") is not False:
    print("FAIL guardrail3-rs.toml must set checks.test = false")
    sys.exit(1)
PY

cargo fmt --check
cargo check
cargo clippy --workspace --all-targets --all-features -- -D warnings
g3rs validate --path . --rules-only

printf 'PASS\n'
