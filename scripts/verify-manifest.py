#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import subprocess
import sys
import tomllib
from pathlib import Path

MANIFEST_PATH = Path(".plans/2026-05-13-150929-goldencheck-architecture.md.manifest.toml")


def load_manifest() -> dict:
    return tomllib.loads(MANIFEST_PATH.read_text())


def fail(lines: list[str]) -> int:
    for line in lines:
        print(line)
    print("FAIL")
    return 1


def pass_layer(name: str) -> int:
    print(f"{name}: PASS")
    return 0


def layer_tree(manifest: dict) -> int:
    missing = [row["path"] for row in manifest.get("tree", []) if not Path(row["path"]).exists()]
    if missing:
        return fail([f"missing path: {path}" for path in missing])
    return pass_layer("layer1 tree")


def path_matches(pattern: str) -> list[str]:
    if pattern == "tests":
        return [
            str(path)
            for path in Path(".").rglob("tests")
            if path.is_dir() and ".git" not in path.parts and "target" not in path.parts
        ]
    return [
        str(path)
        for path in Path(".").rglob(pattern)
        if ".git" not in path.parts and "target" not in path.parts
    ]


def source_files() -> list[Path]:
    roots = [Path("crates"), Path("scripts"), Path("Cargo.toml")]
    files: list[Path] = []
    for root in roots:
        if root.is_file():
            files.append(root)
        elif root.exists():
            files.extend(path for path in root.rglob("*") if path.is_file())
    return files


def layer_forbidden(manifest: dict) -> int:
    findings: list[str] = []
    for row in manifest.get("forbidden_path", []):
        for path in path_matches(row["pattern"]):
            findings.append(f"forbidden path exists: {path}")

    for row in manifest.get("forbidden_source", []):
        pattern = row["pattern"]
        for path in source_files():
            text = path.read_text(errors="ignore")
            if pattern in text:
                findings.append(f"forbidden source '{pattern}' found in {path}")

    if findings:
        return fail(findings)
    return pass_layer("layer2 forbidden")


def get_config_value(config: dict, dotted_key: str):
    value = config
    for part in dotted_key.split("."):
        if not isinstance(value, dict) or part not in value:
            return None
        value = value[part]
    return value


def layer_config(manifest: dict) -> int:
    findings: list[str] = []
    for row in manifest.get("config_value", []):
        file_path = Path(row["file"])
        config = tomllib.loads(file_path.read_text())
        actual = get_config_value(config, row["key"])
        expected = row["value"]
        if actual != expected:
            findings.append(f"{row['file']} {row['key']} expected {expected!r}, got {actual!r}")

    if findings:
        return fail(findings)
    return pass_layer("layer3 config")


def module_name(path: Path) -> str:
    return path.stem


def module_imports(path: Path, module_names: set[str]) -> set[str]:
    text = path.read_text()
    imports: set[str] = set()

    for match in re.finditer(r"\bcrate::([a-zA-Z_][a-zA-Z0-9_]*)", text):
        name = match.group(1)
        if name in module_names:
            imports.add(name)

    for match in re.finditer(r"^\s*(?:pub\(crate\)\s+)?mod\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*;", text, re.MULTILINE):
        name = match.group(1)
        if name in module_names:
            imports.add(name)

    return imports


def layer_modules(manifest: dict) -> int:
    src_dir = Path("crates/goldencheck/src")
    module_files = {module_name(path): path for path in src_dir.glob("*.rs")}
    module_names = set(module_files)
    allowed = {row["from"]: set(row["to"]) for row in manifest.get("module_dep", [])}

    findings: list[str] = []
    for name in sorted(module_names):
        if name not in allowed:
            findings.append(f"module missing from manifest: {name}")
            continue
        actual = module_imports(module_files[name], module_names) - {name}
        unexpected = sorted(actual - allowed[name])
        if unexpected:
            findings.append(f"{name} imports undeclared module(s): {', '.join(unexpected)}")

    for name in sorted(set(allowed) - module_names):
        findings.append(f"manifest module has no source file: {name}")

    if findings:
        return fail(findings)
    return pass_layer("layer4 modules")


STATIC_COMMANDS = {"format", "compile", "clippy", "static-g3rs"}


def run_command(argv: list[str]) -> tuple[int, str]:
    completed = subprocess.run(argv, text=True, capture_output=True, check=False)
    output = completed.stdout + completed.stderr
    return completed.returncode, output


def layer_static(manifest: dict) -> int:
    findings: list[str] = []
    for row in manifest.get("command", []):
        if row["name"] not in STATIC_COMMANDS:
            continue
        code, output = run_command(row["argv"])
        if code != 0:
            findings.append(f"command failed: {row['name']} exit {code}\n{output}")
    if findings:
        return fail(findings)
    return pass_layer("layer5 static")


def layer_goldencheck(manifest: dict) -> int:
    findings: list[str] = []
    commands = [row for row in manifest.get("command", []) if row["name"] == "self-check"]
    if len(commands) != 1:
        return fail(["expected exactly one self-check command"])

    code, output = run_command(commands[0]["argv"])
    if code != 0:
        findings.append(f"self-check failed with exit {code}\n{output}")

    for row in manifest.get("generated_file", []):
        path = Path(row["path"])
        if not path.exists():
            findings.append(f"generated file missing: {path}")

    if findings:
        return fail(findings)
    return pass_layer("layer6 goldencheck")


def layer_cli(manifest: dict) -> int:
    findings: list[str] = []
    for row in manifest.get("cli_command", []):
        code, output = run_command(["cargo", "run", "-p", "goldencheck", "--", row["name"], "--help"])
        if code != 0:
            findings.append(f"cli help failed: {row['name']} exit {code}\n{output}")
            continue
        for flag in row["required_flags"] + row["optional_flags"]:
            if flag not in output:
                findings.append(f"cli command {row['name']} missing flag in help: {flag}")

    if findings:
        return fail(findings)
    return pass_layer("layer7 cli")


LAYERS = {
    "tree": layer_tree,
    "forbidden": layer_forbidden,
    "config": layer_config,
    "modules": layer_modules,
    "static": layer_static,
    "goldencheck": layer_goldencheck,
    "cli": layer_cli,
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("layer", choices=sorted(LAYERS))
    args = parser.parse_args()
    manifest = load_manifest()
    return LAYERS[args.layer](manifest)


if __name__ == "__main__":
    sys.exit(main())
