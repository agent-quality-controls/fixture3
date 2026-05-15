#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
export PYTHONDONTWRITEBYTECODE=1

python3 scripts/verify-ddmin.py
