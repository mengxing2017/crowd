#!/usr/bin/env python3
"""Validate report 35 traceability and record actual check-suite outcomes."""
import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
CHECKS = [
    ("web-and-contract", ["make", "check-web"]),
    ("backend", ["make", "check-backend"]),
    ("isolated-postgres", ["make", "test-business"]),
    ("documentation", ["make", "docs-build"]),
]


def now():
    return datetime.now(timezone.utc).isoformat()


def validate():
    manifest = json.loads((ROOT / "tools/delivery_cases.json").read_text())
    cases = manifest["cases"]
    if [case["id"] for case in cases] != [f"E2E-{n:03}" for n in range(1, 33)]:
        raise ValueError("Delivery manifest must contain E2E-001 through E2E-032 exactly once in order")
    for case in cases:
        if not case["tests"] or case["manual_acceptance"] != "pending":
            raise ValueError(f"{case['id']}: missing test reference or manual acceptance boundary")
        for reference in case["tests"]:
            module, function = reference.split("::")
            if not re.fullmatch(r"[a-z0-9_]+", module + function):
                raise ValueError("Invalid test identifier")
            source = (ROOT / "backend/tests" / f"{module}.rs").read_text()
            if not re.search(r"#\[(?:tokio::)?test\]\s*(?:#\[[^\]]*\]\s*)*(?:async )?fn " + re.escape(function) + r"\(", source, re.S):
                raise ValueError(f"{case['id']}: missing test function {reference}")
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--validate-only", action="store_true")
    parser.add_argument("--output-dir", type=Path)
    args = parser.parse_args()
    manifest = validate()
    print("32 delivery cases and referenced test functions validated", flush=True)
    if args.validate_only:
        return 0
    directory = args.output_dir or Path(tempfile.mkdtemp(prefix="crowd-delivery-"))
    directory.mkdir(parents=True, exist_ok=True)
    evidence_path = directory / "results.json"
    if evidence_path.exists():
        raise ValueError("Evidence already exists; choose a new output directory")
    evidence = {
        "started_at": now(),
        "automated_checks_only": True,
        "manual_acceptance": "pending",
        "cases": manifest["cases"],
        "checks": [{"name": name, "command": command, "status": "not_run"} for name, command in CHECKS],
    }
    def save():
        temporary = directory / "results.json.tmp"
        temporary.write_text(json.dumps(evidence, ensure_ascii=False, indent=2) + "\n")
        temporary.replace(evidence_path)
    save()
    print(f"Evidence: {evidence_path}", flush=True)
    result = 0
    try:
        for check in evidence["checks"]:
            check.update(status="running", started_at=now())
            save()
            code = subprocess.run(check["command"], cwd=ROOT, check=False).returncode
            check.update(status="passed" if code == 0 else "failed", exit_code=code, finished_at=now())
            save()
            if code:
                result = 1
                break
    except (KeyboardInterrupt, OSError) as error:
        # Do not serialize arbitrary process or environment details into delivery artifacts.
        evidence["interrupted"] = type(error).__name__
        result = 1
    finally:
        evidence["finished_at"] = now()
        evidence["status"] = "passed" if not result else "incomplete"
        save()
        print(f"Evidence: {evidence_path}", flush=True)
    return result


if __name__ == "__main__":
    raise SystemExit(main())
