#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys


def main() -> int:
    completed = subprocess.run(
        ["python3", "packages/ddmin/scripts/verify-ddmin.py"],
        text=True,
        check=False,
    )
    return completed.returncode


if __name__ == "__main__":
    sys.exit(main())
