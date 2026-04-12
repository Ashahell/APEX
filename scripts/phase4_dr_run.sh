#!/usr/bin/env bash
set -euo pipefail

BACKUP_DIR="${1:-./backups}"
echo "[phase4] Validating DR runbook tooling..."

for f in backup_db.sh backup_all.sh restore_db.sh restore_all.sh; do
  if [[ ! -x scripts/$f ]]; then
    echo "ERROR: scripts/$f is not executable or missing"; exit 1
  fi
done

echo "[phase4] All required backup/restore scripts exist and are executable. Backups dir: $BACKUP_DIR"
echo "[phase4] DR runbook guidance ready."
