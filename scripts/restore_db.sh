#!/usr/bin/env bash
set -euo pipefail

BACKUP_PATH="$1"
if [[ -z "$BACKUP_PATH" ]]; then
  echo "Usage: restore_db.sh <backup.sql.gz>"; exit 1
fi

echo "[restore] Restoring DB from $BACKUP_PATH to apex_db_prod..."
if [[ "$BACKUP_PATH" == *.gz ]]; then
  gunzip -c "$BACKUP_PATH" | docker exec -i apex_db_prod psql -U apex -d apex
else
  docker exec -i apex_db_prod psql -U apex -d apex < "$BACKUP_PATH"
fi
echo "[restore] DB restoration complete."
