#!/usr/bin/env bash
set -euo pipefail

ROOT_REPO="$(pwd)/.."  # assuming tests/ci_gate_tests is in repo root/tests/ci_gate_tests
REPO_PATH="$(cd "$ROOT_REPO" && pwd)"
SIM_DIR=$(mktemp -d)
echo "[TEST] Creating simulated PR in $SIM_DIR"

git clone "file://$REPO_PATH" "$SIM_DIR/apex-sim" >/dev/null 2>&1
cd "$SIM_DIR/apex-sim"
git fetch origin main >/dev/null 2>&1 || true
git checkout -b feature/ci-gate-sim origin/main

# Trigger cargo audit: modify Cargo.toml in core
echo "# CI gate trigger" >> core/Cargo.toml
git add core/Cargo.toml && git commit -m "ci: trigger cargo audit by adding comment to core/Cargo.toml" >/dev/null 2>&1 || true

# Trigger gateway/npm audit: modify gateway/package.json
echo '"gate_test": true,' >> gateway/package.json
git add gateway/package.json && git commit -m "ci: trigger gateway npm audit by adding test field" >/dev/null 2>&1 || true

# Trigger skills/npm audit: modify skills/package.json
echo '"gate_test": true,' >> skills/package.json
git add skills/package.json && git commit -m "ci: trigger skills npm audit by adding test field" >/dev/null 2>&1 || true

echo "[TEST] Running CI gate script (dry-run to avoid real audits)" 
bash ci/run_security_gates.sh --dry-run --out-dir "$SIM_DIR/reports" --base main
echo "[TEST] Reports available at $SIM_DIR/reports/security_gate_report.json"

exit 0
