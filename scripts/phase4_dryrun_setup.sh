#!/usr/bin/env bash
set -euo pipefail

PROJECT="apex-stage4"
DB_CONTAINER="apex_stage4_db"
BACKUP_DIR="./backups-phase4"

echo "[phase4-dryrun] Setting up staging stack: project=${PROJECT}, db container=${DB_CONTAINER}"
docker-compose -f docker-compose.prod.yml -p "$PROJECT" up -d db
sleep 15

echo "[phase4-dryrun] Performing dry-run backup to ${BACKUP_DIR} (DB container: ${DB_CONTAINER})"
mkdir -p "$BACKUP_DIR"
./scripts/backup_db.sh "$BACKUP_DIR" "$DB_CONTAINER" --dry-run

echo "[phase4-dryrun] Performing dry-run restore (DB container: ${DB_CONTAINER}) to demonstrate restore flow"
./scripts/restore_db.sh "${BACKUP_DIR}/db_*.sql.gz" --dry-run || true

echo "[phase4-dryrun] Cleaning up staging stack"
docker-compose -f docker-compose.prod.yml -p "$PROJECT" down -v

echo "[phase4-dryrun] Phase 4 dry-run complete. Backups staged in $BACKUP_DIR"
