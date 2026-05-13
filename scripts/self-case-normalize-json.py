#!/usr/bin/env python3
from __future__ import annotations

import json
import sys


def main() -> int:
    document = json.loads(sys.stdin.read())
    document["normalized"] = True
    print(json.dumps(document, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
