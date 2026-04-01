#!/usr/bin/env bash
set -euo pipefail

BASE="${BASE_BRANCH:-main}"
DRY_RUN=false
OUT_DIR="ci_artifacts"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      DRY_RUN=true; shift;;
    --out-dir)
      OUT_DIR="$2"; shift 2;;
    --base|--base-branch)
      BASE="$2"; shift 2;;
    *) break;;
  esac
done
echo "[CI] Base branch: ${BASE}"
echo "[CI] Dry-run: ${DRY_RUN}"
echo "[CI] Output dir: ${OUT_DIR}"

mkdir -p "$OUT_DIR"

git fetch origin "${BASE}" --quiet

CHANGED=$(git diff --name-only origin/"${BASE}"...HEAD)
echo "Changed files since ${BASE}:"
echo "$CHANGED"

RUN_CARGO="false"
RUN_NPM_GATEWAY="false"
RUN_NPM_SKILLS="false"
CARGO_CHANGED=0
GATEWAY_CHANGED=0
SKILLS_CHANGED=0
CARGO_STATUS="SKIPPED"
GATEWAY_STATUS="SKIPPED"
SKILLS_STATUS="SKIPPED"
OVERALL="PASS"

if echo "$CHANGED" | grep -E -q '^(Cargo.toml|Cargo.lock)'; then
  RUN_CARGO="true"
  CARGO_CHANGED=1
fi

if echo "$CHANGED" | grep -E -q '^gateway/.*(package.json|package-lock.json|yarn.lock)'; then
  RUN_NPM_GATEWAY="true"
  GATEWAY_CHANGED=1
fi

if echo "$CHANGED" | grep -E -q '^skills/.*(package.json|package-lock.json|yarn.lock)'; then
  RUN_NPM_SKILLS="true"
  SKILLS_CHANGED=1
fi

echo "[CI] Cargo audit needed: ${RUN_CARGO}"
if [ "$DRY_RUN" = "true" ]; then
  echo "[CI] DRY-RUN: Skipping actual cargo audit; assuming PASS"
  CARGO_STATUS="DRY_RUN"
else
  if [ "$RUN_CARGO" = "true" ]; then
    echo "[CI] Running cargo audit..."
    cargo audit -D high
    CARGO_STATUS="PASS"
  fi
fi

if [ "$DRY_RUN" = "true" ]; then
  echo "[CI] DRY-RUN: Skipping npm audits for gateway/skills"
  if [ "$RUN_NPM_GATEWAY" = "true" ]; then GATEWAY_STATUS="DRY_RUN"; fi
  if [ "$RUN_NPM_SKILLS" = "true" ]; then SKILLS_STATUS="DRY_RUN"; fi
else
  if [ "$RUN_NPM_GATEWAY" = "true" ]; then
    echo "[CI] Running npm audit for gateway..."
    (cd gateway && npm ci >/dev/null 2>&1 || true; npm audit --audit-level=high)
    if [ $? -eq 0 ]; then
      GATEWAY_STATUS="PASS"
    else
      GATEWAY_STATUS="FAIL"
      OVERALL="FAIL"
    fi
  else
    GATEWAY_STATUS="SKIPPED"
  fi
  if [ "$RUN_NPM_SKILLS" = "true" ]; then
    echo "[CI] Running npm audit for skills..."
    (cd skills && npm ci >/dev/null 2>&1 || true; npm audit --audit-level=high)
    if [ $? -eq 0 ]; then
      SKILLS_STATUS="PASS"
    else
      SKILLS_STATUS="FAIL"
      OVERALL="FAIL"
    fi
  else
    SKILLS_STATUS="SKIPPED"
  fi
fi

if [ "$DRY_RUN" = "true" ]; then
  OVERALL="DRY_RUN"
fi

REPORT_PATH="${OUT_DIR%/}/security_gate_report.json"
cat > "$REPORT_PATH" <<JSON
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "base_branch": "$BASE",
  "cargo": {"changed": ${CARGO_CHANGED}, "status": "$CARGO_STATUS"},
  "gateway": {"changed": ${GATEWAY_CHANGED}, "status": "$GATEWAY_STATUS"},
  "skills": {"changed": ${SKILLS_CHANGED}, "status": "$SKILLS_STATUS"},
  "overall": "$OVERALL"
}
JSON
echo "[CI] Security gate report written to $REPORT_PATH"

if [ "$DRY_RUN" = "false" ]; then
  if [ "$OVERALL" = "FAIL" ]; then
    exit 1
  fi
fi
