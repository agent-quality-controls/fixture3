#!/usr/bin/env bash
set -euo pipefail

python3 - "$@" <<'PY'
from __future__ import annotations

import json
import sys
from pathlib import Path

records = []
for arg in sorted(sys.argv[1:]):
    path = Path(arg)
    name = path.as_posix()
    if name.endswith("keep/input.json") or name.endswith("keep/support/a.json"):
        source = json.loads(path.read_text())
        records.append(
            {
                "case": source["case"],
                "kind": source["kind"],
                "observed": source["input"].upper(),
            }
        )

print(json.dumps({"count": len(records), "records": records}, indent=2, sort_keys=True))
PY
