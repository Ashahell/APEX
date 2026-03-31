#!/bin/bash
# APEX Production Deployment Script
# Usage: ./deploy.sh [start|stop|restart|backup|restore|status|logs]

set -e

COMPOSE_FILE="docker-compose.yml"
BACKUP_DIR="./backups"
DATA_VOLUME="apex_apex-data"
SOUL_VOLUME="apex_apex-soul"

usage() {
    echo "Usage: $0 {start|stop|restart|backup|restore|status|logs|migrate}"
    echo ""
    echo "Commands:"
    echo "  start     - Start all services"
    echo "  stop      - Stop all services"
    echo "  restart   - Restart all services"
    echo "  backup    - Backup database and soul identity"
    echo "  restore   - Restore from latest backup"
    echo "  status    - Show service status"
    echo "  logs      - Show recent logs"
    echo "  migrate   - Run database migrations"
    exit 1
}

start() {
    echo "Starting APEX services..."
    docker compose -f "$COMPOSE_FILE" up -d
    echo "Waiting for router to be healthy..."
    docker compose -f "$COMPOSE_FILE" exec router curl -sf http://localhost:3000/health >/dev/null 2>&1 || sleep 5
    echo "APEX started successfully!"
    echo "  UI: http://localhost:8083"
    echo "  API: http://localhost:3000"
}

stop() {
    echo "Stopping APEX services..."
    docker compose -f "$COMPOSE_FILE" down
    echo "APEX stopped."
}

restart() {
    stop
    sleep 2
    start
}

backup() {
    echo "Backing up APEX data..."
    mkdir -p "$BACKUP_DIR"
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    BACKUP_PATH="$BACKUP_DIR/apex_backup_$TIMESTAMP"
    mkdir -p "$BACKUP_PATH"

    # Backup database
    docker run --rm -v "$DATA_VOLUME":/data -v "$BACKUP_PATH":/backup \
        alpine cp /data/apex.db /backup/apex.db

    # Backup soul identity
    docker run --rm -v "$SOUL_VOLUME":/soul -v "$BACKUP_PATH":/backup \
        alpine sh -c "cp -r /soul /backup/soul 2>/dev/null || true"

    # Backup config
    cp .env "$BACKUP_PATH/" 2>/dev/null || true

    echo "Backup completed: $BACKUP_PATH"
    echo "Contents:"
    ls -la "$BACKUP_PATH"
}

restore() {
    LATEST=$(ls -t "$BACKUP_DIR" 2>/dev/null | head -1)
    if [ -z "$LATEST" ]; then
        echo "No backups found in $BACKUP_DIR"
        exit 1
    fi

    echo "Restoring from backup: $LATEST"
    BACKUP_PATH="$BACKUP_DIR/$LATEST"

    # Stop services
    stop

    # Restore database
    docker run --rm -v "$DATA_VOLUME":/data -v "$BACKUP_PATH":/backup \
        alpine cp /backup/apex.db /data/apex.db

    # Restore soul identity
    docker run --rm -v "$SOUL_VOLUME":/soul -v "$BACKUP_PATH":/backup \
        alpine sh -c "rm -rf /soul/* && cp -r /backup/soul/* /soul/ 2>/dev/null || true"

    echo "Restore completed from $LATEST"
    echo "Run '$0 start' to restart services"
}

status() {
    echo "APEX Service Status:"
    echo "===================="
    docker compose -f "$COMPOSE_FILE" ps
    echo ""
    echo "Health Check:"
    curl -sf http://localhost:3000/health 2>/dev/null && echo "  Router: ✅ Healthy" || echo "  Router: ❌ Unhealthy"
}

logs() {
    docker compose -f "$COMPOSE_FILE" logs --tail=100 -f router
}

migrate() {
    echo "Running database migrations..."
    docker compose -f "$COMPOSE_FILE" up -d router
    echo "Migrations applied automatically on startup."
}

case "${1:-}" in
    start)    start ;;
    stop)     stop ;;
    restart)  restart ;;
    backup)   backup ;;
    restore)  restore ;;
    status)   status ;;
    logs)     logs ;;
    migrate)  migrate ;;
    *)        usage ;;
esac
