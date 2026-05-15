#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "behavior/cases/contract.json"
PROBE_MANIFEST = ROOT / "scripts/rust-probe/Cargo.toml"


def mask(values: list[int]) -> int:
    result = 0
    for value in values:
        result |= 1 << value
    return result


def members(mask_value: int, candidate_count: int) -> list[int]:
    return [index for index in range(candidate_count) if mask_value & (1 << index)]


def outcomes_for(row: dict) -> str:
    candidate_count = row["candidate_count"]
    interesting_masks = set(row.get("interesting_masks", []))
    unresolved_masks = set(row.get("unresolved_masks", []))
    required = row.get("interesting_when_contains_all")
    values: list[str] = []
    for candidate_mask in range(1 << candidate_count):
        if candidate_mask in unresolved_masks:
            values.append("U")
        elif candidate_mask in interesting_masks:
            values.append("I")
        elif required is not None and all(item in members(candidate_mask, candidate_count) for item in required):
            values.append("I")
        else:
            values.append("N")
    return "".join(values)


def run_probe(lines: list[str]) -> dict[str, dict[str, str]]:
    completed = subprocess.run(
        ["cargo", "run", "-q", "--manifest-path", str(PROBE_MANIFEST)],
        input="\n".join(lines) + "\n",
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        raise RuntimeError(completed.stderr)
    return parse_probe(completed.stdout)


def parse_probe(output: str) -> dict[str, dict[str, str]]:
    parsed: dict[str, dict[str, str]] = {}
    for line in output.splitlines():
        parts = line.split()
        parsed[parts[0]] = dict(part.split("=", 1) for part in parts[1:])
    return parsed


def expected_mask(row: dict, key: str) -> int | None:
    if key not in row:
        return None
    return mask(row[key])


def verify_case(row: dict, result: dict[str, str]) -> list[str]:
    findings: list[str] = []
    remaining = int(result["remaining"])
    expected_remaining = expected_mask(row, "expected_remaining")
    if expected_remaining is not None and remaining != expected_remaining:
        findings.append(f"remaining expected {expected_remaining}, got {remaining}")

    if "expected_remaining_len" in row:
        actual_len = len(members(remaining, row["candidate_count"]))
        if actual_len != row["expected_remaining_len"]:
            findings.append(f"remaining len expected {row['expected_remaining_len']}, got {actual_len}")

    for item in row.get("expected_remaining_contains", []):
        if not remaining & (1 << item):
            findings.append(f"remaining missing expected item {item}")

    if result["guarantee"] != row["expected_guarantee"]:
        findings.append(f"guarantee expected {row['expected_guarantee']}, got {result['guarantee']}")

    if "expected_oracle_calls" in row and int(result["calls"]) != row["expected_oracle_calls"]:
        findings.append(f"oracle calls expected {row['expected_oracle_calls']}, got {result['calls']}")

    minimum_unresolved = row.get("minimum_unresolved_trials")
    if minimum_unresolved is not None and int(result["unresolved"]) < minimum_unresolved:
        findings.append(f"unresolved expected at least {minimum_unresolved}, got {result['unresolved']}")

    return findings


def main() -> int:
    document = json.loads(CONTRACT.read_text())
    lines = []
    for row in document["cases"]:
        max_calls = row.get("max_oracle_calls", "none")
        lines.append(f"{row['name']} {row['candidate_count']} 2 {max_calls} {outcomes_for(row)}")

    results = run_probe(lines)
    findings: list[str] = []
    for row in document["cases"]:
        case_findings = verify_case(row, results[row["name"]])
        findings.extend(f"{row['name']}: {finding}" for finding in case_findings)

    if findings:
        for finding in findings:
            print(finding)
        print("FAIL")
        return 1
    print(f"contract-cases: PASS cases={len(document['cases'])}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
