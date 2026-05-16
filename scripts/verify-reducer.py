#!/usr/bin/env python3
from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RUN_ROOT = ROOT / ".fixture3" / "reducer-run"
SOURCE = ROOT / "examples" / "fake-project"
FIXTURE3 = ROOT / "apps" / "fixtures" / "target" / "debug" / "fixture3"
MANIFESTS = [
    ROOT / ".plans" / "2026-05-15-193357-fixture-reducer-adapter.md.manifest.toml",
    ROOT / ".plans" / "2026-05-15-225212-reducer-runtime-controls.md.manifest.toml",
    ROOT / ".plans" / "2026-05-16-113428-directory-reducer.md.manifest.toml",
    ROOT / ".plans" / "2026-05-16-150547-frontload-help-and-default-reducer-budgets.md.manifest.toml",
]


def run(argv: list[str], cwd: Path) -> tuple[int, str, str]:
    completed = subprocess.run(argv, cwd=cwd, text=True, capture_output=True, check=False)
    return completed.returncode, completed.stdout, completed.stderr


def fail(message: str) -> int:
    print(message)
    print("FAIL")
    return 1


def build_cli() -> tuple[int, str]:
    completed = subprocess.run(
        [
            "cargo",
            "build",
            "--manifest-path",
            str(ROOT / "apps" / "fixtures" / "Cargo.toml"),
            "-p",
            "fixture3-cli",
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    return completed.returncode, completed.stdout + completed.stderr


def reset_run_root() -> Path:
    if RUN_ROOT.exists():
        shutil.rmtree(RUN_ROOT)
    shutil.copytree(SOURCE, RUN_ROOT)
    excluded_root = RUN_ROOT / "behavior" / "fixtures" / "reducer-basic" / "project"
    (excluded_root / ".git").mkdir()
    (excluded_root / ".git" / "ignored.txt").write_text("ignored\n")
    (excluded_root / "target").mkdir()
    (excluded_root / "target" / "ignored.txt").write_text("ignored\n")
    return RUN_ROOT


def read_report(path: Path) -> dict:
    return json.loads(path.read_text())


def verify_manifest_rows() -> list[str]:
    findings: list[str] = []
    for manifest_path in MANIFESTS:
        manifest = tomllib.loads(manifest_path.read_text())
        for row in manifest.get("tree", []):
            if not (ROOT / row["path"]).exists():
                findings.append(f"missing path: {row['path']}")

        for row in manifest.get("manifest_contains", []):
            text = (ROOT / row["file"]).read_text()
            for expected in row["contains"]:
                if expected not in text:
                    findings.append(f"{row['file']} missing text: {expected}")

        for row in manifest.get("cli_command", []):
            code, stdout, stderr = run([str(FIXTURE3), row["name"], "--help"], ROOT)
            output = stdout + stderr
            if code != 0:
                findings.append(f"cli help failed: {row['name']} exit {code}\n{output}")
                continue
            for flag in row["required_flags"] + row["optional_flags"]:
                if flag not in output:
                    findings.append(f"cli command {row['name']} missing flag: {flag}")
            for expected in row.get("help_contains", []):
                if expected not in output:
                    findings.append(f"cli command {row['name']} missing help text: {expected}")

    return findings


def verify_basic(run_root: Path) -> list[str]:
    findings: list[str] = []
    manifest = "behavior/fixtures/reducer-basic/fixture3.yaml"
    fixture_root = "behavior/fixtures/reducer-basic/project"
    work_dir = ".fixture3/reducer-basic-reduce"

    before_files = sorted(
        path.relative_to(run_root / fixture_root).as_posix()
        for path in (run_root / fixture_root).rglob("*")
        if path.is_file()
    )

    code, stdout, stderr = run(
        [
            str(FIXTURE3),
            "reduce",
            "--suite",
            "reducer-basic",
            "--manifest",
            manifest,
            "--fixture-root",
            fixture_root,
            "--work-dir",
            work_dir,
        ],
        run_root,
    )
    if code != 0:
        findings.append(f"reducer-basic exit {code}\nstdout:\n{stdout}\nstderr:\n{stderr}")
        return findings

    try:
        stdout_report = json.loads(stdout)
    except json.JSONDecodeError as error:
        findings.append(f"reducer-basic stdout was not JSON: {error}\n{stdout}")
        return findings

    report_path = run_root / work_dir / "reduce-report.json"
    file_report = read_report(report_path)
    if stdout_report != file_report:
        findings.append("stdout report did not match reduce-report.json")

    expected = {
        "candidate_count": 3,
        "remaining_count": 1,
        "removed_count": 2,
        "guarantee": "complete",
        "remaining_files": ["keep/input.json"],
        "removed_files": ["remove/deep/noise.json", "remove/noise.txt"],
    }
    for key, value in expected.items():
        if file_report.get(key) != value:
            findings.append(f"reducer-basic {key} expected {value!r}, got {file_report.get(key)!r}")

    remaining_text = (run_root / work_dir / "remaining-files.txt").read_text()
    removed_text = (run_root / work_dir / "removed-files.txt").read_text()
    if remaining_text != "keep/input.json\n":
        findings.append(f"unexpected remaining-files.txt: {remaining_text!r}")
    if removed_text != "remove/deep/noise.json\nremove/noise.txt\n":
        findings.append(f"unexpected removed-files.txt: {removed_text!r}")

    after_files = sorted(
        path.relative_to(run_root / fixture_root).as_posix()
        for path in (run_root / fixture_root).rglob("*")
        if path.is_file()
    )
    if after_files != before_files:
        findings.append(f"original fixture root changed: before {before_files!r}, after {after_files!r}")
    if (run_root / work_dir / "trials").exists():
        findings.append("reducer-basic created obsolete trials directory")
    if not (run_root / work_dir / "trial-current").exists():
        findings.append("reducer-basic did not create trial-current")

    return findings


def verify_max_oracle_calls(run_root: Path) -> list[str]:
    findings: list[str] = []
    work_dir = ".fixture3/reducer-basic-limited"
    code, stdout, stderr = run(
        [
            str(FIXTURE3),
            "reduce",
            "--suite",
            "reducer-basic",
            "--manifest",
            "behavior/fixtures/reducer-basic/fixture3.yaml",
            "--fixture-root",
            "behavior/fixtures/reducer-basic/project",
            "--work-dir",
            work_dir,
            "--max-oracle-calls",
            "2",
        ],
        run_root,
    )
    if code != 0:
        findings.append(f"reducer-basic-limited exit {code}\nstdout:\n{stdout}\nstderr:\n{stderr}")
        return findings
    report = json.loads(stdout)
    if report.get("guarantee") != "incomplete:max-oracle-calls-reached":
        findings.append(f"limited guarantee mismatch: {report.get('guarantee')!r}")

    best_report_path = run_root / work_dir / "best" / "reduce-report.json"
    if not best_report_path.exists():
        findings.append("limited reducer did not write best/reduce-report.json")
    else:
        best_report = read_report(best_report_path)
        if best_report.get("guarantee") != "best-so-far":
            findings.append(f"best guarantee mismatch: {best_report.get('guarantee')!r}")
        if not best_report.get("remaining_files"):
            findings.append("best report remaining_files was empty")

    if (run_root / work_dir / "trials").exists():
        findings.append("limited reducer created obsolete trials directory")
    if not (run_root / work_dir / "trial-current").exists():
        findings.append("limited reducer did not create trial-current")

    return findings


def verify_directory_reducer(run_root: Path) -> list[str]:
    findings: list[str] = []
    manifest = "behavior/fixtures/reducer-dirs/fixture3.yaml"
    fixture_root = "behavior/fixtures/reducer-dirs/project"

    before_files = sorted(
        path.relative_to(run_root / fixture_root).as_posix()
        for path in (run_root / fixture_root).rglob("*")
        if path.is_file()
    )

    code, stdout, stderr = run(
        [
            str(FIXTURE3),
            "reduce",
            "--suite",
            "reducer-dirs",
            "--manifest",
            manifest,
            "--fixture-root",
            fixture_root,
            "--work-dir",
            ".fixture3/reducer-dirs-only",
            "--reducers",
            "dirs",
            "--max-oracle-calls",
            "20",
        ],
        run_root,
    )
    if code != 0:
        findings.append(f"reducer-dirs-only exit {code}\nstdout:\n{stdout}\nstderr:\n{stderr}")
        return findings

    report = json.loads(stdout)
    if report.get("reducers") != ["dirs"]:
        findings.append(f"dirs-only reducers mismatch: {report.get('reducers')!r}")
    if "removable-family" not in report.get("removed_directories", []):
        findings.append(f"dirs-only did not remove removable-family: {report.get('removed_directories')!r}")
    for required in ["keep/input.json", "keep/support/a.json", "mixed/keep/input.json"]:
        if required not in report.get("remaining_files", []):
            findings.append(f"dirs-only removed required file: {required}")
    if (run_root / ".fixture3/reducer-dirs-only/trials").exists():
        findings.append("dirs-only reducer created obsolete trials directory")
    if not (run_root / ".fixture3/reducer-dirs-only/trial-current").exists():
        findings.append("dirs-only reducer did not create trial-current")
    if not (run_root / ".fixture3/reducer-dirs-only/best/reduce-report.json").exists():
        findings.append("dirs-only reducer did not write best/reduce-report.json")

    code, stdout, stderr = run(
        [
            str(FIXTURE3),
            "reduce",
            "--suite",
            "reducer-dirs",
            "--manifest",
            manifest,
            "--fixture-root",
            fixture_root,
            "--work-dir",
            ".fixture3/reducer-dirs-default",
        ],
        run_root,
    )
    if code != 0:
        findings.append(f"reducer-dirs-default exit {code}\nstdout:\n{stdout}\nstderr:\n{stderr}")
        return findings

    report = json.loads(stdout)
    if report.get("reducers") != ["dirs", "files"]:
        findings.append(f"default reducers mismatch: {report.get('reducers')!r}")
    if report.get("oracle_calls", 0) > 300:
        findings.append(f"default reducer budget exceeded 300 calls: {report.get('oracle_calls')!r}")
    if report.get("guarantee") != "complete":
        findings.append(f"default reducer guarantee mismatch: {report.get('guarantee')!r}")

    after_files = sorted(
        path.relative_to(run_root / fixture_root).as_posix()
        for path in (run_root / fixture_root).rglob("*")
        if path.is_file()
    )
    if after_files != before_files:
        findings.append(f"directory fixture root changed: before {before_files!r}, after {after_files!r}")

    return findings


def verify_symlink(run_root: Path) -> list[str]:
    findings: list[str] = []
    link = run_root / "behavior" / "fixtures" / "reducer-symlink" / "project" / "link"
    if link.exists() or link.is_symlink():
        link.unlink()
    link.symlink_to("keep/input.json")

    code, stdout, stderr = run(
        [
            str(FIXTURE3),
            "reduce",
            "--suite",
            "reducer-symlink",
            "--manifest",
            "behavior/fixtures/reducer-symlink/fixture3.yaml",
            "--fixture-root",
            "behavior/fixtures/reducer-symlink/project",
            "--work-dir",
            ".fixture3/reducer-symlink-reduce",
        ],
        run_root,
    )
    if code != 2:
        findings.append(f"reducer-symlink expected exit 2, got {code}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    if "symlink fixture content is not supported" not in stderr:
        findings.append(f"reducer-symlink missing symlink error\nstderr:\n{stderr}")
    return findings


def main() -> int:
    code, output = build_cli()
    if code != 0:
        return fail(f"cargo build failed exit {code}\n{output}")

    run_root = reset_run_root()
    findings = []
    findings.extend(verify_manifest_rows())
    findings.extend(verify_basic(run_root))
    findings.extend(verify_max_oracle_calls(run_root))
    findings.extend(verify_directory_reducer(run_root))
    findings.extend(verify_symlink(run_root))
    if findings:
        return fail("\n".join(findings))
    print("reducer: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
