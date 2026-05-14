#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
import tomllib
from pathlib import Path

MANIFEST = Path(".plans/2026-05-14-113743-feature-pipeline-fixtures.md.manifest.toml")
CURRENT_SCAN_ROOTS = [Path("README.md"), Path("AGENTS.md"), Path("crates"), Path("scripts"), Path("behavior"), Path("fixture3.yaml")]


def load_manifest() -> dict:
    return tomllib.loads(MANIFEST.read_text())


def fail(findings: list[str]) -> int:
    for finding in findings:
        print(finding)
    print("FAIL")
    return 1


def files_under(path: Path) -> list[Path]:
    if path.is_file():
        return [path]
    if path.is_dir():
        return [item for item in path.rglob("*") if item.is_file()]
    return []


def run(argv: list[str]) -> tuple[int, str]:
    completed = subprocess.run(argv, check=False, text=True, capture_output=True)
    return completed.returncode, completed.stdout + completed.stderr


def verify_tree(manifest: dict) -> list[str]:
    return [f"missing path: {row['path']}" for row in manifest.get("tree", []) if not Path(row["path"]).exists()]


def verify_forbidden(manifest: dict) -> list[str]:
    findings: list[str] = []
    for row in manifest.get("forbidden_path", []):
        if list(Path(".").glob(row["pattern"])):
            findings.append(f"forbidden path exists: {row['pattern']}")

    files = [file for root in CURRENT_SCAN_ROOTS for file in files_under(root)]
    for row in manifest.get("forbidden_source", []):
        pattern = row["pattern"]
        for file in files:
            if pattern in file.read_text(errors="ignore"):
                findings.append(f"forbidden source '{pattern}' found in {file}")
    return findings


def verify_manifest_contains(manifest: dict) -> list[str]:
    findings: list[str] = []
    for row in manifest.get("manifest_contains", []):
        text = Path(row["file"]).read_text()
        for expected in row["contains"]:
            if expected not in text:
                findings.append(f"{row['file']} missing text: {expected}")
    return findings


def verify_cli(manifest: dict) -> list[str]:
    findings: list[str] = []
    for row in manifest.get("cli_command", []):
        code, output = run(["cargo", "run", "-p", "fixture3-cli", "--", row["name"], "--help"])
        if code != 0:
            findings.append(f"cli help failed: {row['name']} exit {code}\n{output}")
            continue
        for flag in row["required_flags"] + row["optional_flags"]:
            if flag not in output:
                findings.append(f"cli command {row['name']} missing flag: {flag}")
        for text in row.get("help_contains", []):
            if text not in output:
                findings.append(f"cli command {row['name']} missing help text: {text}")
    return findings


def verify_self_cases(manifest: dict) -> list[str]:
    findings: list[str] = []
    for row in manifest.get("self_case", []):
        path = Path("behavior/fixtures/self/cases") / row["name"] / "fixture3.yaml"
        if not path.exists():
            findings.append(f"missing self case manifest: {path}")
    return findings


def main() -> int:
    manifest = load_manifest()
    findings = []
    findings.extend(verify_tree(manifest))
    findings.extend(verify_forbidden(manifest))
    findings.extend(verify_manifest_contains(manifest))
    findings.extend(verify_cli(manifest))
    findings.extend(verify_self_cases(manifest))

    if findings:
        return fail(findings)
    print("feature-pipeline: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
