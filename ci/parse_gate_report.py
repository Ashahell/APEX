#!/usr/bin/env python3
import json, sys
from pathlib import Path

def main():
    report_path = Path('ci_artifacts/security_gate_report.json')
    if not report_path.exists():
        print(f"No report found at {report_path}")
        return 0
    try:
        data = json.loads(report_path.read_text())
    except Exception as e:
        print(f"Failed to parse report: {e}")
        return 1
    cargo = data.get('cargo', {})
    gateway = data.get('gateway', {})
    skills = data.get('skills', {})
    overall = data.get('overall', 'PASS').upper()
    status = 0
    if cargo.get('status', 'SKIPPED') == 'FAIL': status = 1
    if gateway.get('status', 'SKIPPED') == 'FAIL': status = 1
    if skills.get('status', 'SKIPPED') == 'FAIL': status = 1
    # If any FAIL, exit with non-zero; else 0
    sys.exit(status)

if __name__ == '__main__':
    sys.exit(main())
