#!/usr/bin/env bash
set -euo pipefail

BACKUP_DIR="${1:-./backups}"
mkdir -p "$BACKUP_DIR"

echo "[backup] Starting full backup sequence..."
./scripts/backup_db.sh "$BACKUP_DIR"

# Attempt optional memory store backup (best-effort)
echo "[backup] Attempting memory store backup (best effort)"
if docker ps | grep -q apex_router_prod; then
  if docker cp apex_router_prod:/var/lib/memory/memory.sqlite "$BACKUP_DIR/memory_$(date +%F).sqlite" 2>/dev/null; then
    echo "[backup] Memory store backup saved to $BACKUP_DIR/memory_$(date +%F).sqlite"
  else
    echo "[backup] Memory store path not found or inaccessible; skipping memory backup"
  fi
fi
echo "[backup] Backup complete."
