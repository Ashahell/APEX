#!/usr/bin/env bash
set -euo pipefail

BACKUP_DIR="${1:-./backups}"
DB_CONTAINER="${2:-apex_db_prod}"
DRY_RUN=${3:-}
DATE=$(date +%Y%m%d-%H%M%S)
NAME="db_${DATE}.sql.gz"

if [[ "${DRY_RUN}" == "--dry-run" ]]; then
  echo "[backup][DRY-RUN] Would dump from container '$DB_CONTAINER' to '$BACKUP_DIR/$NAME'"
  exit 0
fi

echo "[backup] Dumping Postgres DB '$DB_CONTAINER' (apex) to $BACKUP_DIR/$NAME..."
docker exec -t "$DB_CONTAINER" pg_dump -U apex -d apex | gzip > "$BACKUP_DIR/$NAME"
echo "[backup] DB backup created: $NAME"
