#!/usr/bin/env bash
set -euo pipefail

python3 - "$@" <<'PY'
from __future__ import annotations

import json
import sys
from pathlib import Path

records = []
for arg in sorted(sys.argv[1:]):
    source = json.loads(Path(arg).read_text())
    record = {
        "case": source["case"],
        "kind": source["kind"],
        "observed": source["input"].upper(),
    }
    if "error" in source:
        record["error"] = source["error"]
    records.append(record)

print(json.dumps({"count": len(records), "records": records}, indent=2, sort_keys=True))
PY
