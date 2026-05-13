#!/usr/bin/env python3
from __future__ import annotations

import json
import shutil
import subprocess
import sys
from pathlib import Path


EXPECTED = {
    "approve-no-change": {
        "exit_code": 0,
        "approved_meta_kind": "approved",
        "approved_normalized_matches_received": True,
    },
    "approve-requires-change": {
        "exit_code": 2,
        "stderr_contains": "approve requires --change",
    },
    "approve-with-change": {
        "exit_code": 0,
        "approved_change_path": "behavior/changes/.gitkeep",
        "approved_meta_kind": "approved",
        "approved_normalized_matches_received": True,
    },
    "bad-exit": {
        "exit_code": 2,
        "stderr_contains": "exit code 7 was not in [0]",
    },
    "bad-json": {
        "exit_code": 2,
        "stderr_contains": "json error in command stdout",
    },
    "match": {
        "exit_code": 0,
        "diff_status": "matched",
        "diff_changed": False,
        "stdout_status": "matched",
    },
    "diff-existing": {
        "exit_code": 0,
        "stdout_contains": "status: matched",
    },
    "diff-refresh-mismatch": {
        "exit_code": 1,
        "stdout_contains": "status: different",
    },
    "hash-drift": {
        "exit_code": 2,
        "stderr_contains": "fixture hash changed",
    },
    "init": {
        "exit_code": 0,
        "created_manifest": ".goldencheck/self-init/generated.yaml",
    },
    "mismatch": {
        "exit_code": 1,
        "diff_status": "different",
        "diff_changed": True,
        "stdout_status": "different",
    },
    "missing-approved": {
        "exit_code": 2,
        "stderr_contains": "approved output missing",
    },
    "normalizer": {
        "exit_code": 0,
        "diff_status": "matched",
        "diff_changed": False,
        "stdout_status": "matched",
    },
    "status": {
        "exit_code": 0,
        "stdout_contains": "approved: yes",
    },
}


def read_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    return json.loads(path.read_text())


def line_value(stdout: str, prefix: str) -> str | None:
    for line in stdout.splitlines():
        if line.startswith(prefix):
            return line.removeprefix(prefix).strip()
    return None


def run_command(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, check=False, capture_output=True, text=True)


def check_command(binary: Path, manifest: Path) -> list[str]:
    return [str(binary), "check", "--suite", "case", "--manifest", str(manifest)]


def run_case(binary: Path, manifest: Path) -> dict:
    case = manifest.parent.name
    expected = EXPECTED[case]
    received_root = Path(".goldencheck/self-cases") / case
    prepare_case(case, manifest, received_root)
    result = run_case_command(binary, case, manifest)

    record = {
        "case": case,
        "exit_code": result.returncode,
        "exit_code_ok": result.returncode == expected["exit_code"],
        "received_meta_exists": (received_root / "received.meta.json").exists(),
        "received_normalized_exists": (received_root / "received.normalized.json").exists(),
        "received_raw_exists": (received_root / "received.raw.json").exists(),
    }

    diff_json = read_json(received_root / "diff.json")
    approved_meta = read_json(approved_root(case, manifest) / "approved.meta.json")

    if "stdout_status" in expected:
        record["stdout_status"] = line_value(result.stdout, "status:")
        record["stdout_status_ok"] = record["stdout_status"] == expected["stdout_status"]
    if "stderr_contains" in expected:
        record["stderr_contains_ok"] = expected["stderr_contains"] in result.stderr
    if "stdout_contains" in expected:
        record["stdout_contains_ok"] = expected["stdout_contains"] in result.stdout
    if "created_manifest" in expected:
        record["created_manifest_exists"] = Path(expected["created_manifest"]).exists()
    if "approved_meta_kind" in expected and approved_meta is not None:
        record["approved_meta_kind"] = approved_meta.get("kind")
        record["approved_meta_kind_ok"] = approved_meta.get("kind") == expected["approved_meta_kind"]
    if "approved_change_path" in expected and approved_meta is not None:
        record["approved_change_path"] = approved_meta.get("change_path")
        record["approved_change_path_ok"] = (
            approved_meta.get("change_path") == expected["approved_change_path"]
        )
    if "approved_normalized_matches_received" in expected:
        approved_path = approved_root(case, manifest) / "approved.normalized.json"
        received_path = received_root / "received.normalized.json"
        record["approved_normalized_matches_received"] = (
            approved_path.exists()
            and received_path.exists()
            and approved_path.read_text() == received_path.read_text()
        )
    if diff_json is not None and "diff_changed" in expected:
        record["diff_changed"] = diff_json["changed"]
        record["diff_changed_ok"] = diff_json["changed"] == expected.get("diff_changed")
        record["diff_status"] = diff_json["status"]
        record["diff_status_ok"] = diff_json["status"] == expected.get("diff_status")

    return record


def approved_root(case: str, manifest: Path) -> Path:
    if case.startswith("approve-"):
        return Path(".goldencheck/self-cases") / case / "golden"
    return manifest.parent / "golden"


def prepare_case(case: str, manifest: Path, received_root: Path) -> None:
    if case.startswith("approve-"):
        runtime_golden = approved_root(case, manifest)
        runtime_golden.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(
            manifest.parent / "golden" / "approved.normalized.json",
            runtime_golden / "approved.normalized.json",
        )
        meta = runtime_golden / "approved.meta.json"
        meta.unlink(missing_ok=True)
    if case == "init":
        Path(".goldencheck/self-init/generated.yaml").unlink(missing_ok=True)
    if case != "init":
        for name in ("diff.json", "diff.txt", "received.meta.json", "received.normalized.json", "received.raw.json"):
            (received_root / name).unlink(missing_ok=True)


def run_case_command(binary: Path, case: str, manifest: Path) -> subprocess.CompletedProcess[str]:
    if case == "approve-no-change":
        run_command(check_command(binary, manifest))
        return run_command([str(binary), "approve", "--suite", "case", "--manifest", str(manifest)])
    if case == "approve-requires-change":
        run_command(check_command(binary, manifest))
        return run_command([str(binary), "approve", "--suite", "case", "--manifest", str(manifest)])
    if case == "approve-with-change":
        run_command(check_command(binary, manifest))
        return run_command(
            [
                str(binary),
                "approve",
                "--suite",
                "case",
                "--manifest",
                str(manifest),
                "--change",
                "behavior/changes/.gitkeep",
            ]
        )
    if case == "diff-existing":
        run_command(check_command(binary, manifest))
        return run_command([str(binary), "diff", "--suite", "case", "--manifest", str(manifest)])
    if case == "diff-refresh-mismatch":
        return run_command(
            [str(binary), "diff", "--suite", "case", "--manifest", str(manifest), "--refresh"]
        )
    if case == "init":
        created = Path(".goldencheck/self-init/generated.yaml")
        return run_command([str(binary), "init", "--manifest", str(created)])
    if case == "status":
        run_command(check_command(binary, manifest))
        return run_command([str(binary), "status", "--suite", "case", "--manifest", str(manifest)])
    return run_command(check_command(binary, manifest))


def main() -> int:
    binary = Path("target/debug/goldencheck")
    if not binary.exists():
        print(f"missing binary: {binary}", file=sys.stderr)
        return 2

    manifests = sorted(Path(arg) for arg in sys.argv[1:])
    records = [run_case(binary, manifest) for manifest in manifests]
    print(json.dumps({"cases": records}, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
