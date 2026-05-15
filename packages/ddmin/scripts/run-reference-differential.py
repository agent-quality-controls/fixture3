#!/usr/bin/env python3
from __future__ import annotations

import itertools
import json
import random
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

from reference_ddmin import is_one_minimal, reference_ddmin

ROOT = Path(__file__).resolve().parents[1]
CONFIG = ROOT / "behavior/cases/reference-matrix.json"
PROBE_MANIFEST = ROOT / "scripts/rust-probe/Cargo.toml"


@dataclass(frozen=True)
class Case:
    name: str
    candidate_count: int
    outcomes: str
    category: str


def members(mask: int, candidate_count: int) -> list[int]:
    return [index for index in range(candidate_count) if mask & (1 << index)]


def requires_outcomes(candidate_count: int, required_mask: int) -> str:
    return "".join(
        "I" if candidate_mask & required_mask == required_mask else "N"
        for candidate_mask in range(1 << candidate_count)
    )


def random_outcomes(candidate_count: int, rng: random.Random, include_unresolved: bool) -> str:
    values: list[str] = []
    full = (1 << candidate_count) - 1
    for candidate_mask in range(1 << candidate_count):
        if candidate_mask == 0:
            values.append("N")
        elif candidate_mask == full:
            values.append("I")
        else:
            values.append(rng.choice("INU" if include_unresolved else "IN"))
    return "".join(values)


def make_cases(config: dict) -> list[Case]:
    cases: list[Case] = []
    for candidate_count in config["candidate_counts_for_requires"]:
        for required_mask in range(1 << candidate_count):
            cases.append(
                Case(
                    f"requires-n{candidate_count}-m{required_mask}",
                    candidate_count,
                    requires_outcomes(candidate_count, required_mask),
                    "requires",
                )
            )

    for candidate_count in config["candidate_counts_for_exhaustive_boolean"]:
        variable_masks = [
            candidate_mask
            for candidate_mask in range(1 << candidate_count)
            if candidate_mask not in (0, (1 << candidate_count) - 1)
        ]
        for bits in itertools.product("IN", repeat=len(variable_masks)):
            values = ["N"] * (1 << candidate_count)
            values[(1 << candidate_count) - 1] = "I"
            for candidate_mask, value in zip(variable_masks, bits):
                values[candidate_mask] = value
            cases.append(
                Case(
                    f"boolean-n{candidate_count}-{len(cases)}",
                    candidate_count,
                    "".join(values),
                    "boolean-exhaustive",
                )
            )

    rng = random.Random(config["random_seed"])
    for candidate_count in config["random_candidate_counts"]:
        for index in range(config["random_cases_per_candidate_count"]):
            cases.append(
                Case(
                    f"random-bool-n{candidate_count}-{index}",
                    candidate_count,
                    random_outcomes(candidate_count, rng, False),
                    "random-bool",
                )
            )
            if config["include_unresolved_random_cases"]:
                cases.append(
                    Case(
                        f"random-tri-n{candidate_count}-{index}",
                        candidate_count,
                        random_outcomes(candidate_count, rng, True),
                        "random-tri",
                    )
                )
    return cases


def run_probe(cases: list[Case]) -> dict[str, dict[str, str]]:
    lines = [f"{case.name} {case.candidate_count} 2 none {case.outcomes}" for case in cases]
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


def invariant_failures(case: Case, result: dict[str, str]) -> list[str]:
    findings: list[str] = []
    full = (1 << case.candidate_count) - 1
    remaining = int(result["remaining"])
    removed = int(result["removed"])
    guarantee = result["guarantee"]
    if remaining & ~full:
        findings.append("remaining synthesizes candidate")
    if removed & ~full:
        findings.append("removed synthesizes candidate")
    if remaining & removed:
        findings.append("remaining and removed overlap")
    if remaining | removed != full:
        findings.append("remaining and removed do not cover input")
    if case.outcomes[full] != "I":
        if guarantee != "incomplete:BaselineNotInteresting":
            findings.append("non-interesting baseline did not stop as baseline incomplete")
        return findings
    if guarantee == "complete":
        if case.outcomes[remaining] != "I":
            findings.append("complete output is not interesting")
        if not is_one_minimal(case.outcomes, case.candidate_count, remaining):
            findings.append("complete output is not 1-minimal")
    return findings


def main() -> int:
    config = json.loads(CONFIG.read_text())
    cases = make_cases(config)
    results = run_probe(cases)
    invariant_count = 0
    mismatch_count = 0
    samples: list[str] = []

    for case in cases:
        result = results[case.name]
        case_invariants = invariant_failures(case, result)
        invariant_count += len(case_invariants)
        if case.outcomes[0] == "I":
            continue
        reference_remaining, reference_guarantee = reference_ddmin(case.candidate_count, case.outcomes, 2)
        rust_remaining = int(result["remaining"])
        rust_guarantee = result["guarantee"]
        if (rust_remaining, rust_guarantee) != (reference_remaining, reference_guarantee):
            mismatch_count += 1
            if len(samples) < 20:
                samples.append(
                    f"{case.name}: rust={rust_remaining}/{rust_guarantee} "
                    f"reference={reference_remaining}/{reference_guarantee} "
                    f"rust_1minimal={is_one_minimal(case.outcomes, case.candidate_count, rust_remaining)} "
                    f"reference_1minimal={is_one_minimal(case.outcomes, case.candidate_count, reference_remaining)}"
                )

    findings: list[str] = []
    if len(cases) != config["expected_total_cases"]:
        findings.append(f"total cases expected {config['expected_total_cases']}, got {len(cases)}")
    if invariant_count != config["expected_invariant_failures"]:
        findings.append(
            f"invariant failures expected {config['expected_invariant_failures']}, got {invariant_count}"
        )
    if mismatch_count != config["expected_exact_reference_mismatches"]:
        findings.append(
            f"reference mismatches expected {config['expected_exact_reference_mismatches']}, got {mismatch_count}"
        )
        findings.extend(samples)

    if findings:
        for finding in findings:
            print(finding)
        print("FAIL")
        return 1

    print(f"reference-differential: PASS cases={len(cases)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
