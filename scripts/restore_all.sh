#!/usr/bin/env bash
set -euo pipefail

BACKUP_PATH="${1:-}"
if [[ -z "$BACKUP_PATH" ]]; then
  echo "Usage: restore_all.sh <backup_path>"; exit 1
fi

./scripts/restore_db.sh "$BACKUP_PATH"
echo "[restore] Memory restore (best-effort) not implemented; extend here if you backup memory.sqlite"
