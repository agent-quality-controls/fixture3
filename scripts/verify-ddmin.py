#!/usr/bin/env python3
from __future__ import annotations

import re
import subprocess
import sys
import tomllib
from pathlib import Path

MANIFEST = Path(".plans/2026-05-15-145518-ddmin-package.md.manifest.toml")


def fail(lines: list[str]) -> int:
    for line in lines:
        print(line)
    print("FAIL")
    return 1


def run(argv: list[str]) -> tuple[int, str]:
    completed = subprocess.run(argv, text=True, capture_output=True, check=False)
    return completed.returncode, completed.stdout + completed.stderr


def load_manifest() -> dict:
    return tomllib.loads(MANIFEST.read_text())


def dotted_value(data: dict, key: str):
    value = data
    for part in key.split("."):
        if not isinstance(value, dict) or part not in value:
            return None
        value = value[part]
    return value


def check_tree(manifest: dict) -> list[str]:
    return [
        f"missing path: {row['path']}"
        for row in manifest.get("tree", [])
        if not Path(row["path"]).exists()
    ]


def check_workspace(manifest: dict) -> list[str]:
    findings: list[str] = []
    for row in manifest.get("workspace_member", []):
        workspace = tomllib.loads(Path(row["workspace"]).read_text())
        members = dotted_value(workspace, "workspace.members") or []
        if row["member"] not in members:
            findings.append(f"workspace member missing: {row['workspace']} -> {row['member']}")
    return findings


def check_package(manifest: dict) -> list[str]:
    expected = manifest["package"]
    package_path = Path(expected["path"])
    cargo = tomllib.loads(package_path.read_text())
    workspace = tomllib.loads(Path(expected["workspace"]).read_text())
    package = cargo.get("package", {})
    findings: list[str] = []
    for key in ("name", "license", "publish"):
        actual = resolve_workspace_value(cargo, workspace, f"package.{key}", f"workspace.package.{key}")
        expected_value = expected[key]
        if actual != expected_value:
            findings.append(f"{package_path} package.{key} expected {expected_value!r}, got {actual!r}")
    if package.get("edition") != {"workspace": True}:
        findings.append(f"{package_path} package.edition must use workspace")
    if package.get("rust-version") != {"workspace": True}:
        findings.append(f"{package_path} package.rust-version must use workspace")
    return findings


def resolve_workspace_value(package: dict, workspace: dict, package_key: str, workspace_key: str):
    actual = dotted_value(package, package_key)
    if actual == {"workspace": True}:
        return dotted_value(workspace, workspace_key)
    return actual


def check_public_api(manifest: dict) -> list[str]:
    api = manifest["public_api"]
    source = ddmin_source_text()
    findings: list[str] = []
    for type_name in api["types"]:
        pattern = rf"pub\s+(?:struct|enum|trait)\s+{re.escape(type_name)}\b"
        if not re.search(pattern, source):
            findings.append(f"public type missing: {type_name}")
    for function_name in api["functions"]:
        pattern = rf"pub\s+fn\s+{re.escape(function_name)}\b"
        if not re.search(pattern, source):
            findings.append(f"public function missing: {function_name}")
    return findings


def enum_body(source: str, enum_name: str) -> str | None:
    match = re.search(rf"pub\s+enum\s+{re.escape(enum_name)}\s*\{{(?P<body>.*?)\n\}}", source, re.S)
    if match is None:
        return None
    return match.group("body")


def check_closed_sets(manifest: dict) -> list[str]:
    source = ddmin_source_text()
    findings: list[str] = []
    for row in manifest.get("closed_set", []):
        if row["kind"] != "enum":
            continue
        body = enum_body(source, row["type"])
        if body is None:
            findings.append(f"enum missing: {row['type']}")
            continue
        actual = sorted(set(re.findall(r"^\s*([A-Z][A-Za-z0-9_]*)\b", body, re.M)))
        expected = sorted(row["variants"])
        if actual != expected:
            findings.append(f"enum {row['type']} variants expected {expected}, got {actual}")
    return findings


def ddmin_source_text() -> str:
    src_dir = Path("packages/ddmin/crates/ddmin/src")
    return "\n".join(path.read_text() for path in sorted(src_dir.glob("*.rs")))


def check_behavior(manifest: dict) -> list[str]:
    code, output = run(
        [
            "cargo",
            "run",
            "-q",
            "--manifest-path",
            "packages/ddmin/Cargo.toml",
            "-p",
            "fixture3-ddmin",
            "--example",
            "behavior",
        ]
    )
    if code != 0:
        return [f"ddmin behavior example failed with exit {code}\n{output}"]
    findings: list[str] = []
    for row in manifest.get("behavior_case", []):
        marker = f"case: {row['name']} PASS"
        if marker not in output:
            findings.append(f"behavior case missing output: {marker}")
    if "PASS" not in output.splitlines():
        findings.append("behavior example missing final PASS")
    return findings


def check_commands(manifest: dict) -> list[str]:
    findings: list[str] = []
    for row in manifest.get("command", []):
        code, output = run(row["argv"])
        if code != 0:
            findings.append(f"command failed: {row['name']} exit {code}\n{output}")
    return findings


def main() -> int:
    manifest = load_manifest()
    findings: list[str] = []
    findings.extend(check_tree(manifest))
    findings.extend(check_workspace(manifest))
    findings.extend(check_package(manifest))
    findings.extend(check_public_api(manifest))
    findings.extend(check_closed_sets(manifest))
    findings.extend(check_behavior(manifest))

    if len(sys.argv) > 1 and sys.argv[1] == "--with-commands":
        findings.extend(check_commands(manifest))

    if findings:
        return fail(findings)
    print("ddmin: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
