#!/usr/bin/env bash
set -euo pipefail

ROOT="$1"
if [[ -z "$ROOT" ]]; then
  echo "Usage: restore.sh <backup_dir>"; exit 1
fi

if [[ ! -d "$ROOT" ]]; then
  echo "Backup directory not found: $ROOT"; exit 1
fi

echo "[*] Restoring backup from $ROOT to project root"
cp -r "$ROOT"/* .

echo "[*] Restore complete."
