#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

scripts/verify-layer-1-tree.sh
scripts/verify-layer-2-forbidden.sh
scripts/verify-layer-3-config.sh
scripts/verify-layer-4-modules.sh
scripts/verify-layer-5-static.sh
scripts/verify-layer-6-fixture3.sh
scripts/verify-layer-7-cli.sh

printf 'PASS\n'
