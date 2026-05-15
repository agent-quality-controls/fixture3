#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tomllib
from pathlib import Path

MANIFEST = Path(".plans/2026-05-15-160906-rust-workspace-split.md.manifest.toml")
WORKSPACES = [
    Path("apps/fixtures"),
    Path("apps/fixture3-install"),
    Path("packages/ddmin"),
]


def load_manifest() -> dict:
    return tomllib.loads(MANIFEST.read_text())


def run(argv: list[str]) -> tuple[int, str]:
    completed = subprocess.run(argv, text=True, capture_output=True, check=False)
    return completed.returncode, completed.stdout + completed.stderr


def fail(findings: list[str]) -> int:
    for finding in findings:
        print(finding)
    print("FAIL")
    return 1


def cargo_packages(workspace: Path) -> tuple[int, set[str], str]:
    code, output = run(
        [
            "cargo",
            "metadata",
            "--format-version",
            "1",
            "--manifest-path",
            str(workspace / "Cargo.toml"),
            "--no-deps",
        ]
    )
    if code != 0:
        return code, set(), output
    metadata = json.loads(output)
    return 0, {package["name"] for package in metadata["packages"]}, ""


def verify_manifest() -> list[str]:
    manifest = load_manifest()
    findings: list[str] = []

    for row in manifest.get("tree", []):
        if not Path(row["path"]).exists():
            findings.append(f"missing path: {row['path']}")

    for row in manifest.get("forbidden_path", []):
        if Path(row["path"]).exists():
            findings.append(f"forbidden path exists: {row['path']}")

    for row in manifest.get("workspace", []):
        workspace = Path(row["path"])
        cargo = workspace / "Cargo.toml"
        guardrail = workspace / "guardrail3-rs.toml"
        if not cargo.exists() or not guardrail.exists():
            findings.append(f"workspace marker pair missing: {workspace}")
            continue
        parsed = tomllib.loads(cargo.read_text())
        if "workspace" not in parsed:
            findings.append(f"workspace table missing: {cargo}")
            continue
        code, packages, output = cargo_packages(workspace)
        if code != 0:
            findings.append(f"cargo metadata failed for {workspace}\n{output}")
            continue
        expected = set(row["packages"])
        if packages != expected:
            findings.append(f"{workspace} packages expected {sorted(expected)}, got {sorted(packages)}")

    return findings


def run_for_workspaces(kind: str, before_manifest: list[str], after_manifest: list[str] | None = None) -> list[str]:
    findings: list[str] = []
    after = after_manifest or []
    for workspace in WORKSPACES:
        code, output = run(before_manifest + [str(workspace / "Cargo.toml")] + after)
        if code != 0:
            findings.append(f"{kind} failed for {workspace} exit {code}\n{output}")
    return findings


def run_g3rs() -> list[str]:
    findings: list[str] = []
    code, output = run(["g3rs", "validate", "repo"])
    if code != 0:
        findings.append(f"g3rs validate repo failed exit {code}\n{output}")
    for workspace in WORKSPACES:
        code, output = run(
            ["g3rs", "validate", "workspace", "--path", str(workspace), "--rules-only"]
        )
        if code != 0:
            findings.append(f"g3rs validate failed for {workspace} exit {code}\n{output}")
    return findings


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--fmt", action="store_true")
    parser.add_argument("--clippy", action="store_true")
    parser.add_argument("--g3rs", action="store_true")
    args = parser.parse_args()

    findings = verify_manifest()
    if args.check:
        findings.extend(run_for_workspaces("cargo check", ["cargo", "check", "--manifest-path"]))
    if args.fmt:
        findings.extend(
            run_for_workspaces("cargo fmt", ["cargo", "fmt", "--manifest-path"], ["--all", "--check"])
        )
    if args.clippy:
        findings.extend(
            run_for_workspaces(
                "cargo clippy",
                [
                    "cargo",
                    "clippy",
                    "--manifest-path",
                ],
                ["--workspace", "--all-targets", "--all-features", "--", "-D", "warnings"],
            )
        )
    if args.g3rs:
        findings.extend(run_g3rs())

    if findings:
        return fail(findings)
    print("rust-workspaces: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
