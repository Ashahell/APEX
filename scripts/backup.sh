#!/usr/bin/env bash
set -euo pipefail

DATE=$(date +%Y%m%d-%H%M%S)
BACKUP_DIR="$(pwd)/backups/$DATE"
mkdir -p "$BACKUP_DIR"

echo "[*] Starting backup to $BACKUP_DIR"
git ls-files -z | xargs -0 -I{} bash -c '[[ -f {} ]] && cp --parents {} "$BACKUP_DIR"/ || true'

echo "[*] Backup complete. Archive at $BACKUP_DIR"
