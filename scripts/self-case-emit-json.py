#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path


def main() -> int:
    records = []
    for arg in sorted(sys.argv[1:]):
        path = Path(arg)
        records.append({"path": arg, "text": path.read_text()})
    print(json.dumps({"fixtures": records}, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
